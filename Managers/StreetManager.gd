class_name StreetManager
extends EntityManager

# ==============================================================================

const MIN_ANGLE = 0.785398 # 45°
const MAX_CONNECTIONS_PER_INTERSECTION = 4
const MIN_LENGTH = 50
const MAX_LENGTH = 2000

const SNAP_DISTANCE = 25

const STREET_GROUP = "Streets"

# ==============================================================================

var Street = preload("res://Street.gd")
var Intersection = preload("res://Intersection.gd")
var District = preload("res://District.gd")

# ==============================================================================

signal street_count_changed(count)

signal street_created(street)

signal street_deleted(street)

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _district_manager = get_node("../DistrictManager")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _camera = get_node("../Camera2D")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/HBoxContainer/MainPanel")

# ==============================================================================

enum State {NOTHING, START_STREET, END_STREET}
var state = State.NOTHING

var _starting_street
var _starting_intersection
var _valid_street = false

var enabled = false

var temp = { "start": Vector2(0, 0), "end": Vector2(0,0)}

var temp_street : Line2D = null



# ==============================================================================


func preload_entity(data):
	var street = Street.new()

	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	street.update()
	
	add_child(street)
	
	# must be after add_child in order to overwrite id
	street.set_id(data.id)

func load_entity(data):
	var street = get_by_id(data.id)
		
	street.set_start(_intersection_manager.get_by_id(data.start))
	street.set_end(_intersection_manager.get_by_id(data.end))	
	
	if data.d_l as float >= 0:
		street.left_district =  _district_manager.get_by_id(data.d_l as float)
	if data.d_r as float  >= 0:
		street.right_district =  _district_manager.get_by_id(data.d_r as float)
		
func delete(street):
	street.start.remove_street(street)
	street.end.remove_street(street)
	
	emit_signal("street_deleted", street)
	
	.delete(street)
	
	emit_signal("street_count_changed", get_all().size())
	
# Called when the node enters the scene tree for the first time.
func _ready():	
	entity_group = STREET_GROUP
	_gui_main_panel.connect("street_changed", self, "_change_temp_street")
	_gui_main_panel.connect("destroy_mode_changed", self, "_enable_destroy")
	
	
	temp_street = Line2D.new()
	temp_street.width = Street.WIDTH * 2
	temp_street.default_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
	temp_street.points.resize(2)
	add_child(temp_street)
	
func _enable_destroy(value):
	if value:
		enabled = false
	
func _change_temp_street(street : Buildable):
	if street:
		enabled = true
	else:
		enabled = false
	
func _update_temp_street_start(position):
	_starting_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
		
	if _starting_intersection:
		state = State.START_STREET
		temp["start"] = _starting_intersection.position
		temp["end"] = _starting_intersection.position
	else:
		state = State.START_STREET
		
		_starting_street = _starts_on_street(position)
						
		if _starting_street:					
			var lambda = ((_starting_street.norm.x * (position.x - _starting_street.start.position.x)) + 
						  (_starting_street.norm.y * (position.y - _starting_street.start.position.y)))
						
			position = Vector2((_starting_street.norm.x * lambda) + _starting_street.start.position.x, 
							   (_starting_street.norm.y * lambda) + _starting_street.start.position.y)
							
		temp["start"] = position
		temp["end"] = position
		
		temp_street.points[0] = position		
		temp_street.points[1] = position		
				
func _place_street():
	if _valid_street:
		create_street(temp["start"], temp["end"])
	else:				
		temp["start"] = Vector2(0, 0)
		temp["end"] = Vector2(0, 0)
		
	_starting_street = null
	_starting_intersection = null
		
	state = State.NOTHING		
		
func _update_temp_end(position):
	var near_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
	if near_intersection:
		position = near_intersection.position


	if state == State.START_STREET:
		temp["end"] = position
		
		if (_exceeds_min_angle(temp["start"], temp["end"]) and 
			not _violates_max_streets_on_intersection(temp["start"], temp["end"]) and 
			not _violates_min_length(temp["start"], temp["end"]) and
			not _violates_max_length(temp["start"], temp["end"]) and
			not _violates_intersecting_another_street()):
			
			temp_street.default_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
			_valid_street = true
		else:
			temp_street.default_color = Color.red
			_valid_street = false
			
	temp_street.points = [temp["start"], temp["end"]]
	temp_street.update()
					
func _input(event):	
	._input(event)
	
	if not enabled:
		return
		
	if event is InputEventMouseButton:
		if event.is_action_pressed("place_object"):		
			_update_temp_street_start(_mouse_world_position)
	
		if event.is_action_released("place_object") and state == State.START_STREET:
			_place_street()
		
	if event is InputEventMouseMotion:
		_update_temp_end(_mouse_world_position)
		
		
func is_point_on_street(pt : Vector2) -> Street :
	for s in get_tree().get_nodes_in_group(STREET_GROUP):
		if Geometry.is_point_in_polygon(pt, s.global_polygon()):
			return s
			
	return null
				
func create_street(start_pos : Vector2, end_pos : Vector2):
	var start = _intersection_manager.is_near_intersection(start_pos, SNAP_DISTANCE)
	var end = _intersection_manager.is_near_intersection(end_pos, SNAP_DISTANCE)
	
	var split_start : Street = null
	var split_end : Street = null
	if not start:
		split_start = is_point_on_street(start_pos)
		
		if not split_start:
			start = _intersection_manager.create_intersection(start_pos)
		
	if not end:
		split_end = is_point_on_street(end_pos)
				
		if not split_end:
			end = _intersection_manager.create_intersection(end_pos)
			
	if not split_start and split_end:
		var _start = split_end.start
		var _end = split_end.end
		split_end.get_parent().remove_child(split_end)
		
		var intersection = _intersection_manager.create_intersection(end_pos)
		_create_street(_start, intersection)	
		_create_street(start, intersection)
		_create_street(intersection, _end)		
	
		return
		
	if split_start and not split_end:
		var _start = split_start.start
		var _end = split_start.end
		split_start.get_parent().remove_child(split_start)
		
		var intersection = _intersection_manager.create_intersection(start_pos)
		_create_street(_start, intersection)	
		_create_street(intersection, end)
		_create_street(intersection, _end)
		
		return
		
		
	if split_start and split_end:
		var _split_end_start = split_end.start
		var _split_end_end = split_end.end
		
		var _split_start_start = split_start.start
		var _split_start_end = split_start.end
					
		#var district = split_end.get_district(split_end.get_side_of_point(end_pos))	
		var left_district = split_start.get_district(District.Side.LEFT)
		var right_district = split_start.get_district(District.Side.RIGHT)
		
		if left_district:
			_district_manager.delete(left_district)
		if right_district:
			_district_manager.delete(right_district)

		
		split_end.get_parent().remove_child(split_end)
		split_start.get_parent().remove_child(split_start)

		var intersection_start = _intersection_manager.create_intersection(start_pos)
		var intersection_end = _intersection_manager.create_intersection(end_pos)
		
		var new_street = _create_street(intersection_end, intersection_start)
		
		_create_street(_split_end_start, intersection_end)	
		_create_street(intersection_end, _split_end_end)
					
		
		_create_street(_split_start_start, intersection_start)	
		_create_street(intersection_start, _split_start_end)			
		
		_district_manager._create_district_on_side(new_street, District.Side.LEFT)
		_district_manager._create_district_on_side(new_street, District.Side.RIGHT)			
		
		emit_signal("street_count_changed", get_all().size())
		
		return
	
	var street = _create_street(start, end)
	emit_signal("street_created", street)
	
	
	emit_signal("street_count_changed", get_all().size())
				

func _starts_on_street(point):
	for node in get_tree().get_nodes_in_group(STREET_GROUP):
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null
		
func _create_street(start_intersection : Intersection, end_intersection : Intersection) -> Street:
	var street = Street.new()
	street.set_start(start_intersection)
	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	add_child(street)
	
	street.set_end(end_intersection)	
	
	return street

func _exceeds_min_angle(start, end):
	
	if _starting_intersection:
		var prev_angle = _starting_intersection.previous_angle_to_line(end)
		var next_angle = _starting_intersection.next_angle_to_line(end)
		
		if prev_angle < MIN_ANGLE:
			return false
		if next_angle < MIN_ANGLE:
			return false
			
	return true

func _violates_min_length(start, end):
	return start.distance_to(end) < MIN_LENGTH

func _violates_max_length(start, end):
	return start.distance_to(end) > MAX_LENGTH

func _violates_max_streets_on_intersection(start, end):
	if _starting_intersection:
		return _starting_intersection._streets.size() >= MAX_CONNECTIONS_PER_INTERSECTION

func _violates_intersecting_another_street():
	if (_intersection_manager.is_near_intersection(temp_street.points[0], SNAP_DISTANCE) or
		_intersection_manager.is_near_intersection(temp_street.points[1], SNAP_DISTANCE)):
	
		return false
			
	for street in get_all():
		if Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, temp_street.points[0], temp_street.points[1]):
			return true
	
	return false


func _intersect_with_street(position):
	var near_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
	if near_intersection:
		temp["end"] = near_intersection.position
						
	var a = []
	for s in get_tree().get_nodes_in_group(STREET_GROUP):
		if _starting_street and s.get_index() == _starting_street.get_index():
			continue;
			
		var r = Geometry.segment_intersects_segment_2d(s.position, s.end.position, temp["start"], temp["end"])
		
		if r:
			a.append(r)


	var shortest = Vector2(50000, 50000)
	var shortest_distance = 80000000000

	if a:				
		for intersection in a:				
			var distance = intersection.distance_squared_to(temp["start"]) 
			if distance < shortest_distance:
				shortest_distance = distance
				shortest = intersection

		temp["end"] = shortest
