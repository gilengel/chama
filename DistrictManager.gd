class_name DistrictManager
extends EntityManager

onready var _street_manager = get_node("../StreetManager")
onready var _intersection_manager = get_node("../IntersectionManager")

const DISTRICT_GROUP = "Districts"

var _outer_boundary : PoolVector2Array = []

signal district_count_changed(count)

# Called when the node enters the scene tree for the first time.
func _ready():
	entity_group = DISTRICT_GROUP
	_street_manager.connect("street_created", self, "_update_districts_for_street")
	_intersection_manager.connect("intersection_created", self, "_update_district_outer_boundary")
	
func _update_district_outer_boundary(intersection : Intersection):
	var pts = []
	for intersection in _intersection_manager.get_all():
		pts.append(intersection.position)

	
	_outer_boundary = Geometry.convex_hull_2d(pts)
	#_outer_boundary = Geometry.convex_hull_2d(pts)
	
func _district_is_outer(points: PoolVector2Array):
	# special case for the first district
	if get_all().empty():
		return false
		
	var polygon_size = points.size()
	for i in range(_outer_boundary.size()):
		var found = false
		for j in range(polygon_size):
			if points[j].is_equal_approx(_outer_boundary[i]):
				found = true
				break
		
		if not found: 
			return false
			
	return true
	
		
func _create_district_on_side(street: Street, side: int):
	assert(side >= 0 and side <= 1)
		
	var temp_district = enclosed(street, side)
	
	if temp_district.enclosed and not _district_is_outer(temp_district.points):
		var district = create_district(temp_district.points)
		
		for street_and_side in temp_district.streets:
			street_and_side.street.set_district(district, street_and_side.side)
			
			var other_side = District.Side.LEFT if street_and_side.side == District.Side.RIGHT else District.Side.RIGHT
			var neighbouring_district = street_and_side.street.get_district(other_side)
			
			if neighbouring_district:
				neighbouring_district.neighbours.append(district)
				neighbouring_district.update()
				district.neighbours.append(neighbouring_district)
	
func _update_districts_for_street(street: Street):
	if street.end._streets.size() == 1 or street.start._streets.size() == 1:
		return
	
	_create_district_on_side(street, District.Side.LEFT)
	_create_district_on_side(street, District.Side.RIGHT)

func enclosed(start: Street, side : int):
	var next = start.get_next(side)
	var street = start		
	var forward = true
	
	var streets = []	
	var points = []
	var i = 0
	while next != start and next:
		streets.append({ "street" : street, "side": side})
		
		if forward:
			next = street.get_next(side)

			points.append(street.start.position)
		else:
			next = street.get_previous(side)
			
		
			points.append(street.end.position)
			
		if next and (street.end == next.end or street.start == next.start):	
			forward = !forward
			
			side = District.Side.LEFT if side == District.Side.RIGHT else District.Side.RIGHT
				

		street = next
	
	return { "enclosed": next == start, "streets": streets, "points": points }

func preload_entity(data):
	var district = District.new()

	var pts = []
	for i in range(0, data.pts.size(), 2):
		pts.append(Vector2(data.pts[i], data.pts[i+1]))
	
	district.set_points(pts)
	district.add_to_group(DISTRICT_GROUP)
	district.add_to_group($"../".PERSIST_GROUP)
	add_child(district)
	district.set_id(data.id)

func load_entity(data):
	var district = get_by_id(data.id)
	for i in data.neighbours:
		district.neighbours.append(get_by_id(i))

func create_district(points: PoolVector2Array):
	var district = District.new()

	district.set_points(points)
	district.add_to_group(DISTRICT_GROUP)
	district.add_to_group($"../".PERSIST_GROUP)
	add_child(district)
	
	emit_signal("district_count_changed", get_all().size())

	return district


func remove_district_via_street(street: Street, side: int) -> void:
	var temp_district = enclosed(street, side)
	
	if street.get_district(side):
		street.get_district(side).queue_free()
	
	if temp_district:
		for s in temp_district.streets:
			s.street.set_district(null, s.side)
