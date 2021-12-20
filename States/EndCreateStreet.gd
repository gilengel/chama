extends State

# ==============================================================================

onready var _intersection_manager = get_node("../../IntersectionManager")
onready var _street_manager = get_node("../../StreetManager")
onready var _district_manager = get_node("../../DistrictManager")

# ==============================================================================

var _starting_street
var _starting_intersection
var _valid_street

var _splitted_starting_streets

var _end
var _temp_end

# ==============================================================================

const SNAP_DISTANCE = 25



# ==============================================================================

signal street_created(street)

# ==============================================================================

var temp_street : Street = null

# ==============================================================================

func _starts_on_street(point):
	for node in _street_manager.get_all():
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null
	

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
		var street = _create_street(start, intersection)
		_create_street(intersection, _end)	
		
		emit_signal("street_created", street)
	
		return
		
	if split_start and not split_end:
		var _start = split_start.start
		var _end = split_start.end
			
		
		var intersection = _intersection_manager.create_intersection(start_pos)
		split_start.end = intersection
		var street = _create_street(intersection, end)
		_create_street(intersection, _end)

		emit_signal("street_created", street)
		
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
		
		var street = _create_street(intersection_end, intersection_start)
		
		_create_street(_split_end_start, intersection_end)	
		_create_street(intersection_end, _split_end_end)					
		
		_create_street(_split_start_start, intersection_start)	
		_create_street(intersection_start, _split_start_end)	
		
		emit_signal("street_created", street)		
		
		return
	
	
	var street = _create_street(start, end)
	emit_signal("street_created", street)

		
func _create_street(start_intersection : Intersection, end_intersection : Intersection) -> Street:
	var street = _street_manager.create()
	street.set_start(start_intersection)
	street.set_end(end_intersection)
	
	return street




	
func _update_temp_end(position):
	
	var near_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE, [temp_street.start])

	if near_intersection and temp_street.end != near_intersection:
		position = near_intersection.position
		
		_end = near_intersection
		temp_street.end = _end		
	else: 		
		_end = _temp_end		
		_end.position = position
				
	temp_street._update_geometry()
	
	temp_street.visible = temp_street.length >= 80
	
	# necessary to get the changes of the street propagated to the intersection
	temp_street.end.update()
	temp_street.start.update()
	
	var s = temp_street.end.global_position
	var e = temp_street.start.global_position
	
	var angles = temp_street.start.get_angles_to_adjacent_streets(temp_street)
	
	print(angles)
	for i in range(1):
		if angles[i] > -Street.MIN_ANGLE and angles[i] < Street.MIN_ANGLE:

			var norms = temp_street.start.get_norm_of_adjacent_streets(temp_street)
			var length = temp_street.length

			temp_street.end.global_position = temp_street.start.global_position + norms[i].rotated(deg2rad(45.0 if angles[i] <= 0 else -45)) * length
		


	if temp_street.is_constructable():
		temp_street.normal_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
		_valid_street = true
	else:
		temp_street.normal_color = Color.red
		_valid_street = false


	
# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseButton:
		if _event.is_action_released("place_object") and temp_street.is_constructable():
			var start_pos = temp_street.start.global_position 
			var end_pos = temp_street.end.global_position
			
			# corner case if starting and ending street are splitted
			var end_street = _street_manager.is_point_on_street(end_pos)
			if not _splitted_starting_streets.empty() and end_street:
				_district_manager.delete_at_position(temp_street.global_position + temp_street.norm * (temp_street.length * 0.5))
			#	_district_manager.delete(end_street.right_district)
				
			_street_manager.delete(temp_street)
			
			create_street(start_pos, end_pos)			
			
			state_machine.transition_to("StartCreateStreet")
			
		if _event.is_action_released("cancel_action"):
			if not _splitted_starting_streets.empty():
				_splitted_starting_streets[0].end = _splitted_starting_streets[1].end
				_street_manager.delete(_splitted_starting_streets[1])
				
			_street_manager.delete(temp_street)
			
			state_machine.transition_to("StartCreateStreet")
		
	if _event is InputEventMouseMotion:
		_update_temp_end(_mouse_world_position)
		temp_street.start.update()


# Virtual function. Called by the state machine upon changing the active state. The `msg` parameter
# is a dictionary with arbitrary data the state can use to initialize itself.
func enter(_msg := {}) -> void:
	assert(_msg.has("start_position"))
	
	_splitted_starting_streets = _msg.start_splitted

	temp_street = _msg.street
	
	_end = temp_street.end
	_temp_end = _end
	#temp_street = Line2D.new()
	#temp_street.width = Street.WIDTH * 2
	temp_street.color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
	add_child(temp_street)
	
	#temp_street.points = [_msg.start_position, _msg.start_position]
