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
var _points
var control_points : Array = []

# ==============================================================================

const SNAP_DISTANCE = 25

const SEGMENTS = 2

# ==============================================================================

signal street_created(street)

# ==============================================================================

var temp_streets : Array = []

# ==============================================================================

func _quadratic_bezier(p0: Vector2, p1: Vector2, p2: Vector2):
	var points = []
	
	var factor = 1.0 / SEGMENTS
	for i in range(SEGMENTS + 1):
		var t = i * factor
				
		var q0 = p0.linear_interpolate(p1, t)
		var q1 = p1.linear_interpolate(p2, t)

		var r0 = q0.linear_interpolate(q1, t)

		points.push_back(r0.linear_interpolate(r0, t))
	
	return points
	
func _get_intersection_with_another_street(end):
	for street in _street_manager.get_all(temp_streets):
		var ignore = false
		for j in _splitted_starting_streets:
			if street.get_id() == j.get_id():
				ignore = true
				
		if ignore:
			continue
		
		var intersection = Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, temp_streets.back().start.global_position, end)
		
		if intersection and intersection != street.start.global_position and intersection != street.end.global_position:
			return { "street": street, "intersection": intersection }

	return null	
	
func snap_temp_end_position(raw_position : Vector2) -> Vector2:
	var new_position = Vector2(0, 0)
	
	var near_intersections = _intersection_manager.get_near_intersections(raw_position, SNAP_DISTANCE)
	near_intersections.erase(temp_streets.back().end)
	near_intersections.erase(temp_streets.back().start)

	if not near_intersections.empty():
		new_position = near_intersections.front().position
		
	
	var street_interaction = _get_intersection_with_another_street(raw_position)
	
	
	if street_interaction:	
		var start_distance = street_interaction.intersection.distance_to(street_interaction.street.start.position)
		var end_distance = street_interaction.intersection.distance_to(street_interaction.street.end.position)

		if start_distance < SNAP_DISTANCE:
			new_position = street_interaction.street.start.position
		elif end_distance < SNAP_DISTANCE:
			new_position = street_interaction.street.end.position
		else:					
			new_position = street_interaction.intersection
	else:	
		new_position = raw_position
		
	var s = temp_streets.back().end.global_position
	var e = temp_streets.back().start.global_position

	var angles = temp_streets.back().start.get_angles_to_adjacent_streets(temp_streets.back())

	var temp_end = temp_streets.back().end.position 
	temp_streets.back().end.position = new_position
	for i in range(2):
		if angles[i] > -Street.MIN_ANGLE and angles[i] < Street.MIN_ANGLE:
			

			var norms = temp_streets.back().start.get_norm_of_adjacent_streets(temp_streets.back())
			var length = temp_streets.back().length

			new_position = temp_streets.back().start.global_position + norms[i].rotated(0.78539833793 if angles[i] <= 0 else -0.78539833793) * length
	
	temp_streets.back().end.position = temp_end

	return new_position		
	
func _starts_on_street(point):
	for node in _street_manager.get_all():
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null
	

func create_street(start_pos : Vector2, end_pos : Vector2, width : float, points: Array):
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
		_create_street(intersection, _end, width)	
		var streets = _create_curved_street(start, intersection, width, points)	
		for street in streets:
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
		
	var streets = _create_curved_street(start, end, width, points)
	for street in streets:
		emit_signal("street_created", street)
		

func _create_curved_street(start_intersection : Intersection, end_intersection : Intersection, width : float, points: Array) -> Array:
	var streets = []

	# Corner case for linear streets
	if SEGMENTS == 1:
		return [_create_street(start_intersection, end_intersection, width)]
		
	var end = _intersection_manager.create()
	end.position = points[1]
	var previous = _create_street(start_intersection, end, width)
	
	for i in range(2, points.size()):
		var start = previous.end
		end = end_intersection if i == points.size() -1 else _intersection_manager.create()
		end.position = points[i]

		var street = _create_street(start, end, width)

		streets.push_back(street)

		previous = street
	
	return streets
	
func _create_street(start_intersection : Intersection, end_intersection : Intersection, width : float) -> Street:
	var street = _street_manager.create()
	street.set_start(start_intersection)
	street.set_end(end_intersection)
	street.width = width
	
	return street


func _update_temp_end(position):
	var near_intersections = _intersection_manager.get_near_intersections(position, SNAP_DISTANCE)
	for s in temp_streets:
		near_intersections.erase(s.end)
		near_intersections.erase(s.start)

	if not near_intersections.empty():
		temp_streets.back().end = near_intersections.front()
		
	
	var street_interaction = _get_intersection_with_another_street(position)
	
	
	if street_interaction:	
		var start_distance = street_interaction.intersection.distance_to(street_interaction.street.start.position)
		var end_distance = street_interaction.intersection.distance_to(street_interaction.street.end.position)

		if start_distance < SNAP_DISTANCE:
			#_temp_end = temp_street.end
			temp_streets.back().end = street_interaction.street.start		
		elif end_distance < SNAP_DISTANCE:
			#_temp_end = temp_street.end	
			temp_streets.back().end = street_interaction.street.end
		else:					
			var _end = temp_streets.back().end
			temp_streets.back().end = _temp_end
			
			_end.remove_street(temp_streets.back().end)

				
			temp_streets.back().end.position = street_interaction.intersection
	else:
		if temp_streets.back().end != _temp_end:
			temp_streets.back().end = _temp_end
			
		temp_streets.back().end.position = position




# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	
	if _event is InputEventMouseButton:
		if _event.is_action_released("place_object"):
			control_points.push_back(_mouse_world_position)

			_points = _quadratic_bezier(control_points[0], control_points[1], control_points[1])
			for i in range(1, _points.size()-1):
				var street = _street_manager.create()
				var start = temp_streets.back().end

				street.start = start
				street.end = _intersection_manager.create()
				street.end.position = _points[i]
				
				temp_streets.push_back(street)
			_temp_end = temp_streets.back().end
			
		if _event.is_action_pressed("place_object"):							
			if _temp_end != temp_streets.back().end:
				_intersection_manager.delete(_temp_end)	
				
			for segment in temp_streets:
				_street_manager.delete(segment)
				
			var width = temp_streets.front().width	
			create_street(temp_streets.front().start.position, temp_streets.back().end.position, width, _points)
			
			state_machine.transition_to("StartCreateStreet", { "street": "CurvedStreet" })
	
		if _event.is_action_released("cancel_action"):
			if _temp_end != temp_streets.back().end:
				_intersection_manager.delete(_temp_end)
				
			for street in temp_streets:
				_street_manager.delete(street)
				
			if not _splitted_starting_streets.empty():
				_splitted_starting_streets[0].end = _splitted_starting_streets[1].end
				_street_manager.delete(_splitted_starting_streets[1])
			

				
			
			state_machine.transition_to("StartCreateStreet", { "street": "CurvedStreet" })
		
	if _event is InputEventMouseMotion:
		var position = _mouse_world_position# snap_temp_end_position(_mouse_world_position)
		if control_points.size() == 2:	
			_update_temp_end(_mouse_world_position)
			temp_streets.front().start.update()
			
			position = temp_streets.back().end.position
			
			_points = _quadratic_bezier(control_points[0], control_points[1], position)
					
			for i in range(1, _points.size()-1):
				temp_streets[i-1].end.position = _points[i]
				temp_streets[i].start.position = _points[i]
		
		
		temp_streets.back().update()


func enter(_msg := {}) -> void:
	assert(_msg.has("start_position"))
	
	_splitted_starting_streets = _msg.start_splitted
	
	temp_streets.clear()
	temp_streets.push_back(_msg.street)
	
	_temp_end = temp_streets.back().end
			

	control_points.clear()
	control_points.push_back(temp_streets.back().start.position)
