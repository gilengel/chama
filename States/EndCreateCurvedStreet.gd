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

var control_points : Array = []

# ==============================================================================

const SNAP_DISTANCE = 25

const SEGMENTS = 10

# ==============================================================================

signal street_created(street)

# ==============================================================================

var temp_streets : Array = []

# ==============================================================================

func _quadratic_bezier(p0: Vector2, p1: Vector2, p2: Vector2):
	var points = []
	
	var factor = 1.0 / SEGMENTS
	for i in range(SEGMENTS):
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
	

func create_street(start_pos : Vector2, end_pos : Vector2, width : float):
	var start = _intersection_manager.is_near_intersection(start_pos, SNAP_DISTANCE, [temp_streets.front().start])
	var end = _intersection_manager.is_near_intersection(end_pos, SNAP_DISTANCE, [temp_streets.back().end])
	
	
	
	var split_start : Street = null
	var split_end : Street = null
	if not start:
		split_start = _street_manager.is_point_on_street(start_pos, [temp_streets.front(), temp_streets.back()])
	else:
		var _start = temp_streets.front().start
		temp_streets.front().start = start
		_intersection_manager.delete(_start)
		
	if not end:
		split_end = _street_manager.is_point_on_street(end_pos, [temp_streets.back()])
	else:
		var _end = temp_streets.back().end
		temp_streets.back().end = end
		_intersection_manager.delete(_end)
		
		
			
	if not split_start and split_end:
		
		var _start = split_end.start
		var _end = split_end.end


		#var intersection = _intersection_manager.create_intersection(end_pos)
		split_end.end = temp_streets.back().end
		#var street = _create_street(start, intersection, width)
		_create_street(temp_streets.back().end, _end, width)	
		
		#emit_signal("street_created", street)
		return
		
	if split_start and not split_end:
		var _start = split_start.start
		var _end = split_start.end


		var intersection = _intersection_manager.create_intersection(start_pos)
		#split_start.end = intersection
		#var street = _create_street(intersection, end, width)
		#_create_street(intersection, _end, width)

		#emit_signal("street_created", street)
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

		return
		
func _create_street(start_intersection : Intersection, end_intersection : Intersection, width : float) -> Street:
	var street = _street_manager.create()
	street.set_start(start_intersection)
	street.set_end(end_intersection)
	street.width = width
	
	return street




# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseButton:
		if _event.is_action_released("place_object"):
			control_points.push_back(_mouse_world_position)
			
			var points = _quadratic_bezier(control_points[0], control_points[1], control_points[1])
			for i in range(1, points.size()):
				var street = _street_manager.create()
				var start = temp_streets.back().end

				street.start = start
				street.end = _intersection_manager.create()
				street.end.position = points[i]
				
				temp_streets.push_back(street)
				
			
			
		if _event.is_action_pressed("place_object"): #and temp_street.is_constructable():
			
			create_street(temp_streets.front().start.position, temp_streets.back().end.position, 10)
			
			#for segment in temp_streets:
			emit_signal("street_created", temp_streets.back())
			
			
			state_machine.transition_to("StartCreateStreet", { "street": "CurvedStreet" })
#			control_points.push_back(_mouse_world_position)
#
#			var points = _quadratic_bezier(control_points[0], control_points[1], control_points[2])
#			for i in range(1, points.size()):
#				temp_streets[i-1].end.position = points[i]
#				temp_streets[i].start.position = points[i]
#
#			temp_streets.back().end.position = control_points[2]
			
		if _event.is_action_released("cancel_action"):
			for street in temp_streets:
				_street_manager.delete(street)
			state_machine.transition_to("StartCreateStreet", { "street": "CurvedStreet" })
		
	if _event is InputEventMouseMotion:
		var position = snap_temp_end_position(_mouse_world_position)
		if control_points.size() == 2:			
			var points = _quadratic_bezier(control_points[0], control_points[1], position)
			
			for i in range(1, points.size()):
				temp_streets[i-1].end.position = points[i]
				temp_streets[i].start.position = points[i]
				
				
	
		temp_streets.back().end.position = position
		
		
		temp_streets.back().update()


func enter(_msg := {}) -> void:
	assert(_msg.has("start_position"))
	
	_splitted_starting_streets = _msg.start_splitted
	
	temp_streets.clear()
	temp_streets.push_back(_msg.street)
	
	print(_splitted_starting_streets)
			

	control_points.clear()
	control_points.push_back(temp_streets.back().start.position)
