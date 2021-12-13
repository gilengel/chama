extends State

# ==============================================================================

onready var _intersection_manager = get_node("../../IntersectionManager")
onready var _street_manager = get_node("../../StreetManager")
onready var _district_manager = get_node("../../DistrictManager")

# ==============================================================================

var _starting_street
var _starting_intersection
var _valid_street

# ==============================================================================

const SNAP_DISTANCE = 25

const MIN_ANGLE = 0.785398 # 45°
const MAX_CONNECTIONS_PER_INTERSECTION = 4
const MIN_LENGTH = 50
const MAX_LENGTH = 2000

# ==============================================================================

signal street_created(street)

# ==============================================================================

var temp_street : Line2D = null

# ==============================================================================

func _starts_on_street(point):
	for node in _street_manager.get_all():
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null
	
func recreate_districts(street: Street):
	assert(street.left_district == null)
	assert(street.right_district == null)
	
	_district_manager.create_districts_for_street(street)

func create_street(start_pos : Vector2, end_pos : Vector2):
	var start = _intersection_manager.is_near_intersection(start_pos, SNAP_DISTANCE)
	var end = _intersection_manager.is_near_intersection(end_pos, SNAP_DISTANCE)
	
	var split_start : Street = null
	var split_end : Street = null
	if not start:
		split_start = _street_manager.is_point_on_street(start_pos)
		
		if not split_start:
			start = _intersection_manager.create_intersection(start_pos)
		
	if not end:
		split_end = _street_manager.is_point_on_street(end_pos)
				
		if not split_end:
			end = _intersection_manager.create_intersection(end_pos)
			
	if not split_start and split_end:
		
		var _start = split_end.start
		var _end = split_end.end


		var intersection = _intersection_manager.create_intersection(end_pos)
		split_end.end = intersection	
		_create_street(start, intersection)
		_create_street(intersection, _end)	
	
		return
		
	if split_start and not split_end:
		var _start = split_start.start
		var _end = split_start.end
			
		
		var intersection = _intersection_manager.create_intersection(start_pos)
		split_start.end = intersection
		_create_street(intersection, end)
		_create_street(intersection, _end)

		
		return
		
		
	if split_start and split_end:
		var _split_end_start = split_end.start
		var _split_end_end = split_end.end
		
		var _split_start_start = split_start.start
		var _split_start_end = split_start.end					

		_street_manager.delete(split_start)	
		_street_manager.delete(split_end)	

		var intersection_start = _intersection_manager.create_intersection(start_pos)
		var intersection_end = _intersection_manager.create_intersection(end_pos)
		
		var new_street = _create_street(intersection_end, intersection_start)
		
		_create_street(_split_end_start, intersection_end)	
		_create_street(intersection_end, _split_end_end)					
		
		_create_street(_split_start_start, intersection_start)	
		_create_street(intersection_start, _split_start_end)			
		
		return
	
	
	var street = _create_street(start, end)
	emit_signal("street_created", street)

		
func _create_street(start_intersection : Intersection, end_intersection : Intersection) -> Street:
	var street = _street_manager.create()
	street.set_start(start_intersection)
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
			
	for street in _street_manager.get_all():
		if Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, temp_street.points[0], temp_street.points[1]):
			return true
	
	return false
	
func _update_temp_end(position):
	var near_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
	if near_intersection:
		position = near_intersection.position

	temp_street.points[1] = position
	
	var s = temp_street.points[0]
	var e = temp_street.points[1]
	if (_exceeds_min_angle(s, e) and 
		not _violates_max_streets_on_intersection(s, e) and 
		not _violates_min_length(s, e) and
		not _violates_max_length(s, e) and
		not _violates_intersecting_another_street()):

		temp_street.default_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
		_valid_street = true
	else:
		temp_street.default_color = Color.red
		_valid_street = false

	

	
# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseButton:
		if _event.is_action_released("place_object"):
			if _valid_street:
				create_street(temp_street.points[0], temp_street.points[1])
				temp_street.queue_free()
			else:
				temp_street.queue_free()
			
			state_machine.transition_to("StartCreateStreet")
			
		if _event.is_action_released("cancel_action"):
			temp_street.queue_free()
			state_machine.transition_to("StartCreateStreet")
		
	if _event is InputEventMouseMotion:
		_update_temp_end(_mouse_world_position)


# Virtual function. Called by the state machine upon changing the active state. The `msg` parameter
# is a dictionary with arbitrary data the state can use to initialize itself.
func enter(_msg := {}) -> void:
	assert(_msg.has("start_position"))

	temp_street = Line2D.new()
	temp_street.width = Street.WIDTH * 2
	temp_street.default_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
	add_child(temp_street)
	
	temp_street.points = [_msg.start_position, _msg.start_position]
