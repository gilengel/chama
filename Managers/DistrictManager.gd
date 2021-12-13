class_name DistrictManager
extends EntityManager

onready var _street_manager = get_node("../StreetManager")
onready var _intersection_manager = get_node("../IntersectionManager")

onready var state_end_create_street = get_node("../BuildingStateMachine/EndCreateStreet")

const DISTRICT_GROUP = "Districts"

var _outer_boundary : PoolVector2Array = []

signal district_count_changed(count)

func delete(entity):
	var district = entity as District
	
	for neighbour in district.neighbours:
		# neighbour is already pending to be killed or invalid so no
		# further action required
		if not is_instance_valid(neighbour):
			continue
			
		neighbour.neighbours.erase(district)
		
	.delete(entity)
	emit_signal("district_count_changed", get_all().size())

# Called when the node enters the scene tree for the first time.
func _ready():
	entity_group = DISTRICT_GROUP
	_street_manager.connect("street_created", self, "_update_districts_for_street")
	_street_manager.connect("street_deleted", self, "_delete_districts_for_street")
	_intersection_manager.connect("intersection_count_changed", self, "_update_district_outer_boundary")
	
	state_end_create_street.connect("street_created", self, "create_districts_for_street")
	
func _update_district_outer_boundary(size = 0):
	var pts = []
	for intersection in _intersection_manager.get_all():
		pts.append(intersection.position)

	
	_outer_boundary = Geometry.convex_hull_2d(pts)
	
func _district_is_outer(points: PoolVector2Array):
	# special case for the first district
	if get_all().empty():
		return false
		
	var polygon_size = points.size()
	var outer_size = _outer_boundary.size()
	
	for i in range(_outer_boundary.size()):
		var found = false
		for j in range(polygon_size):
			if points[j].is_equal_approx(_outer_boundary[i]):
				found = true
				break
		
		if not found: 
			return false
			
	return true
	
func create_districts_for_street(street: Street):
	
	var left = enclosed(street, District.Side.LEFT)
	var left_center = ExtendedGeometry.average_centroid_polygon_2d(left.points)
	var is_left = street.get_side_of_point(left_center) == District.Side.LEFT
	
	if is_left and left.enclosed and enclosed_area_is_free(left):
		var district = create_district(left.points)

		for street_and_side in left.streets:
			street_and_side.street.set_district(district, street_and_side.side)

			var other_side = District.Side.LEFT if street_and_side.side == District.Side.RIGHT else District.Side.RIGHT
			var neighbouring_district = street_and_side.street.get_district(other_side)

			if is_instance_valid(neighbouring_district):
				neighbouring_district.neighbours.append(district)
				neighbouring_district.update()
				district.neighbours.append(neighbouring_district)		
	
	var right = enclosed(street, District.Side.RIGHT)
	var right_center = ExtendedGeometry.average_centroid_polygon_2d(right.points)
	var is_right = street.get_side_of_point(right_center) == District.Side.RIGHT

	if is_right and right.enclosed and enclosed_area_is_free(right):
		var district = create_district(right.points)

		for street_and_side in right.streets:
			street_and_side.street.set_district(district, street_and_side.side)

			var other_side = District.Side.LEFT if street_and_side.side == District.Side.RIGHT else District.Side.RIGHT
			var neighbouring_district = street_and_side.street.get_district(other_side)

			if is_instance_valid(neighbouring_district):
				neighbouring_district.neighbours.append(district)
				neighbouring_district.update()
				district.neighbours.append(neighbouring_district)	







	
#func _create_district_on_side(street: Street, side: int):
#	assert(side >= 0 and side <= 1)
#
#	var temp_district = enclosed(street, side)
#
#	if temp_district.enclosed and not _district_is_outer(temp_district.points):
#		var district = create_district(temp_district.points)
#
#		for street_and_side in temp_district.streets:
#			street_and_side.street.set_district(district, street_and_side.side)
#
#			var other_side = District.Side.LEFT if street_and_side.side == District.Side.RIGHT else District.Side.RIGHT
#			var neighbouring_district = street_and_side.street.get_district(other_side)
#
#			if is_instance_valid(neighbouring_district):
#				neighbouring_district.neighbours.append(district)
#				neighbouring_district.update()
#				district.neighbours.append(neighbouring_district)
	
#func _update_districts_for_street(street: Street):
#	if street.end._streets.size() == 1 or street.start._streets.size() == 1:
#		return
#
#	_create_district_on_side(street, District.Side.LEFT)
#	_create_district_on_side(street, District.Side.RIGHT)

func _delete_districts_for_street(street: Street):
	for side in [District.Side.LEFT, District.Side.RIGHT]:
			
		var enclosed = enclosed(street, side)
		if enclosed.enclosed:
			for i in range(enclosed.streets.size()):
				enclosed.streets[i].set_district(null, enclosed.streets[i].side)
		
		var district = street.get_district(side)
		if is_instance_valid(district):
			delete(district)

	
func enclosed_area_is_free(ring: Dictionary) -> bool:
	for street_side_pair in ring.streets:
		if street_side_pair.street.get_district(street_side_pair.side):
			return false
	
	return true
	
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
	
	_update_district_outer_boundary()
	
	emit_signal("district_count_changed", get_all().size())

	return district


func remove_district_via_street(street: Street, side: int) -> void:
	var temp_district = enclosed(street, side)
	
	if street.get_district(side):
		street.get_district(side).queue_free()
	
	if temp_district:
		for s in temp_district.streets:
			s.street.set_district(null, s.side)
