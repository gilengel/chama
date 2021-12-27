extends State

# ==============================================================================

onready var _intersection_manager = get_node("../../IntersectionManager")
onready var _street_manager = get_node("../../StreetManager")
onready var _district_manager = get_node("../../DistrictManager")
onready var _style_manager = get_node("../../StyleManager")

# ==============================================================================

var _starting_street
var _starting_intersection
var _valid_street

var _splitted_starting_streets

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
	

func create_street(start_pos : Vector2, end_pos : Vector2, width : float):
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
		var street = _create_street(start, intersection, width)
		_create_street(intersection, _end, width)	
		
		emit_signal("street_created", street)
	
		return
		
	if split_start and not split_end:
		var _start = split_start.start
		var _end = split_start.end
			
		
		var intersection = _intersection_manager.create_intersection(start_pos)
		split_start.end = intersection
		var street = _create_street(intersection, end, width)
		_create_street(intersection, _end, width)

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
		
		var street = _create_street(intersection_end, intersection_start, width)
		
		_create_street(_split_end_start, intersection_end, width)	
		_create_street(intersection_end, _split_end_end, width)					
		
		_create_street(_split_start_start, intersection_start, width)	
		_create_street(intersection_start, _split_start_end, width)	
		
		emit_signal("street_created", street)		
		
		return
	
	
	var street = _create_street(start, end, width)
	emit_signal("street_created", street)

		
func _create_street(start_intersection : Intersection, end_intersection : Intersection, width : float) -> Street:
	var street = _street_manager.create()
	street.set_start(start_intersection)
	street.set_end(end_intersection)
	street.width = width
	
	return street



func _get_intersection_with_another_street(end):
	for street in _street_manager.get_all():
		var ignore = false if street.get_id() != temp_street.get_id() else true
		for j in _splitted_starting_streets:
			if street.get_id() == j.get_id():
				ignore = true
				
		if ignore:
			continue
		
		var intersection = Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, temp_street.start.global_position, end)
		
		if intersection and intersection != street.start.global_position and intersection != street.end.global_position:
			return { "street": street, "intersection": intersection }

	return null
	
func _update_temp_end(position):
	#temp_street.end.position = position
	

	var near_intersections = _intersection_manager.get_near_intersections(position, SNAP_DISTANCE)
	near_intersections.erase(temp_street.end)
	near_intersections.erase(temp_street.start)

	if not near_intersections.empty():
		temp_street.end = near_intersections.front()
		
	
	var street_interaction = _get_intersection_with_another_street(position)
	
	
	if street_interaction:	
		var start_distance = street_interaction.intersection.distance_to(street_interaction.street.start.position)
		var end_distance = street_interaction.intersection.distance_to(street_interaction.street.end.position)
		#print("%s %s" % [start_distance, end_distance])
		
		if start_distance < SNAP_DISTANCE:
			#_temp_end = temp_street.end
			temp_street.end = street_interaction.street.start		
		elif end_distance < SNAP_DISTANCE:
			#_temp_end = temp_street.end	
			temp_street.end = street_interaction.street.end
		else:					
			var _end = temp_street.end
			temp_street.end = _temp_end
			
			_end.remove_street(temp_street)

				
			temp_street.end.position = street_interaction.intersection
	else:
		if temp_street.end != _temp_end:
			temp_street.end = _temp_end
			
		temp_street.end.position = position

		

				
	temp_street._update_geometry()
	
	#temp_street.visible = temp_street.length >= 80
	
	var s = temp_street.end.global_position
	var e = temp_street.start.global_position

	var angles = temp_street.start.get_angles_to_adjacent_streets(temp_street)

	for i in range(2):
		if angles[i] > -Street.MIN_ANGLE and angles[i] < Street.MIN_ANGLE:
			

			var norms = temp_street.start.get_norm_of_adjacent_streets(temp_street)
			var length = temp_street.length

			temp_street.end.global_position = temp_street.start.global_position + norms[i].rotated(0.78539833793 if angles[i] <= 0 else -0.78539833793) * length



	if temp_street.is_constructable():
		temp_street.normal_color = _style_manager.get_color(StyleManager.Colors.Street)
		_valid_street = true
	else:
		temp_street.normal_color = _style_manager.get_color(StyleManager.Colors.Error)
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
			
			var width = temp_street.width
			_street_manager.delete(temp_street)
			
			create_street(start_pos, end_pos, width)			
			
			state_machine.transition_to("StartCreateStreet")
			
		if _event.is_action_released("cancel_action"):
			if not _splitted_starting_streets.empty():
				_splitted_starting_streets[0].end = _splitted_starting_streets[1].end
				_street_manager.delete(_splitted_starting_streets[1])
			
			if _temp_end != temp_street.end:
				_intersection_manager.delete(_temp_end)
				
			_street_manager.delete(temp_street)
			
			
			
			state_machine.transition_to("StartCreateStreet")
		
	if _event is InputEventMouseMotion:
		_update_temp_end(_mouse_world_position)
		temp_street.start.update()


func enter(_msg := {}) -> void:
	assert(_msg.has("start_position"))
	
	_splitted_starting_streets = _msg.start_splitted

	temp_street = _msg.street
	
	_temp_end = temp_street.end
	add_child(temp_street)
