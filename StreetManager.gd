class_name StreetManager
extends Node

# ==============================================================================

const MIN_ANGLE = 0.785398 # 45°
const MAX_CONNECTIONS_PER_INTERSECTION = 4
const MIN_LENGTH = 50
const MAX_LENGTH = 2000

const STREET_GROUP = "Streets"

# ==============================================================================

var Street = preload("res://Street.gd")
var Intersection = preload("res://Intersection.gd")
var District = preload("res://District.gd")

# ==============================================================================

signal street_count_changed(count)

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _district_manager = get_node("../DistrictManager")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _camera = get_node("../Camera2D")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/MainPanel")

# ==============================================================================

enum State {NOTHING, START_STREET, END_STREET}
var state = State.NOTHING

var _starting_street
var _starting_intersection
var _valid_street = false

var enabled = true

var temp = { "start": Vector2(0, 0), "end": Vector2(0,0)}

# ==============================================================================


func preload_street(data):
	var street = Street.new()

	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	street.update()
	
	add_child(street)
	
	# must be after add_child in order to overwrite id
	street.set_id(data.id)

func load_street(data):
	var street = get_street_by_id(data.id)
		
	var start = _intersection_manager.get_intersection_by_id(data.start)	
	var end = _intersection_manager.get_intersection_by_id(data.end)
		
	#street.set_previous()
	
	street.set_start(start)
	street.set_end(end)	
	
func get_streets():
	return get_tree().get_nodes_in_group(STREET_GROUP)
	
static func sort_ascending(a : Dictionary, b : Dictionary):
	if a.distance < b.distance:
		return true
		
	return false	
	
func get_closest_streets_to(point : Vector2) -> Dictionary:
	var closest_street = null
	var closest_distance = 1000000000
	
	var _streets = []
	for street in get_streets():
		var pt = Geometry.get_closest_point_to_segment_2d(point, street.global_position, street.end.global_position)
		_streets.append({ "distance": point.distance_to(pt), "street": street})

	_streets.sort_custom(self, "sort_ascending")
	

	return { 
		"street" : _streets[0].street, 
		"side" : _streets[0].street.get_side_of_point(point) 
	}
		
func get_street_by_id(id) -> Street:
	for node in get_tree().get_nodes_in_group(STREET_GROUP):
		if node.get_id() == id:
			return node
	
	return null


# Called when the node enters the scene tree for the first time.
func _ready():	
	_gui_main_panel.connect("street_changed", self, "_change_temp_street")
	
func _change_temp_street(street : Buildable):
	if street:
		enabled = true
	else:
		enabled = false
	
func _update_temp_street_start(position):
	_starting_intersection = _intersection_manager.is_near_intersection(position, 50)
		
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

	if state == State.START_STREET:
		temp["end"] = position
		
		if (_exceeds_min_angle(temp["start"], temp["end"]) and 
			not _violates_max_streets_on_intersection(temp["start"], temp["end"]) and 
			not _violates_min_length(temp["start"], temp["end"]) and
			not _violates_max_length(temp["start"], temp["end"])):
			_valid_street = true
		else:
			_valid_street = false
					
func _input(event):	
	if not enabled:
		return
		
	if event is InputEventMouseButton:
		if event.is_action_pressed("place_object"):		
			_update_temp_street_start(event.position + _camera.offset)
	
		if event.is_action_released("place_object") and state == State.START_STREET:
			_place_street()
		
	if event is InputEventMouseMotion:
		_update_temp_end(event.position + _camera.offset)
		
		
func _create_intersection(position):
	var intersection = Intersection.new()

	intersection.position = position
	intersection.add_to_group(_intersection_manager.INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)	
	
	return intersection	
	
func _is_point_on_street(pt : Vector2) -> Street :
	for s in get_tree().get_nodes_in_group(STREET_GROUP):
		if Geometry.is_point_in_polygon(pt, s.global_polygon()):
			return s
			
	return null
				
func create_street(start_pos : Vector2, end_pos : Vector2):
	var start = _intersection_manager.is_near_intersection(start_pos, 50)
	var end = _intersection_manager.is_near_intersection(end_pos, 50)
	
	var split_start : Street = null
	var split_end : Street = null
	if not start:
		split_start = _is_point_on_street(start_pos)
		
		if not split_start:
			start = _intersection_manager.create_intersection(start_pos)
		
	if not end:
		split_end = _is_point_on_street(end_pos)
				
		if not split_end:
			end = _intersection_manager.create_intersection(end_pos)
	
	if split_start:
		var _start = split_start.start
		var _end = split_start.end
		split_start.get_parent().remove_child(split_start)
		
		var intersection = _intersection_manager.create_intersection(start_pos)
		var s1 = _create_street(_start, intersection)	
		var s2 = _create_street(intersection, end)
		var s3 = _create_street(intersection, _end)
		
		emit_signal("street_count_changed", get_streets().size())
		
		return
	
	_create_street(start, end)
	
	emit_signal("street_count_changed", get_streets().size())
				



	
#func create_street(start_pos, end_pos):
#	var street = Street.new()
#
#	var start = _is_near_intersection(start_pos, 50)
#	var splitted_start
#	var splitted_end
#
#	if not start:		
#		for s in get_tree().get_nodes_in_group(STREET_GROUP):
#			if Geometry.is_point_in_polygon(start_pos, s.global_polygon()):
#				start = _split_street_on_start(s, start_pos, end_pos)				
#				splitted_start = s
#
#		if not splitted_start:
#			start = _create_intersection(start_pos)		
#
#	var end = _is_near_intersection(end_pos, 50)
#	if not end:
#		for s in get_tree().get_nodes_in_group(STREET_GROUP):
#			if Geometry.is_point_in_polygon(end_pos, s.global_polygon()):
#				print(":)")
#				#end = _split_street_on_end(s, end_pos, start_pos, start)	
#				#splitted_end = s
#
#		if not splitted_end:
#			end = _create_intersection(end_pos)
#
#	if not splitted_start and not splitted_end:
#		street.set_start(start)
#		street.set_end(end)		
#
#		street.add_to_group(STREET_GROUP)
#		street.add_to_group($"../".PERSIST_GROUP)
#		add_child(street)	
#
#		var previous = street.start.previous_street(street)
#		street.set_previous(previous)
#
#		if previous:
#			print("%s -> %s %s" % [street.get_id(), previous.get_id(), previous.end.previous_street(previous).get_id()])
#			previous.set_next(previous.end.previous_street(previous))
#
#		var next = street.end.next_street(street)
#		street.set_next(next)
#
#		if next and not next.get_previous():
#			next.set_previous(street)	
#
#
#		street.set_left_district(_district_manager.create_district(street, District.Side.LEFT))
#		street.set_right_district(_district_manager.create_district(street, District.Side.RIGHT))
#
#
#	emit_signal("street_count_changed", get_streets().size())
#
#	return street
	

	
func _is_near_street(point):
	for street in get_tree().get_nodes_in_group(STREET_GROUP):
		if self.street and street.get_index() == self.street.get_index():
			continue;
			
		if Geometry.is_point_in_polygon(point, street.global_polygon()):
			var intersection = Geometry.line_intersects_line_2d(street.start.position, street.norm, point,  Vector2(-street.norm.y, street.norm.x))
			return [street, intersection]
			
	
func is_near_street(point: Vector2) -> Street:
	for node in get_tree().get_nodes_in_group(STREET_GROUP):
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null


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
		
func _remove_street(street : Street):
	street.get_left_district().queue_free()
	street.get_right_district().queue_free()
	
	street.queue_free()
	
	
func _split_street_on_start(street, split_point, end_point):
	var start = street.start
	
	var crossroad = Intersection.new()
	crossroad.position = split_point
	crossroad.add_to_group(_intersection_manager.INTERSECTION_GROUP)
	crossroad.add_to_group($"../".PERSIST_GROUP)
	add_child(crossroad)
	
	_remove_street(street)
	street = _create_street(start, crossroad)
	
	street.set_left_district(_district_manager.create_district(street, District.Side.LEFT))
	street.set_right_district(_district_manager.create_district(street, District.Side.RIGHT))
	
	
	
	
	street = _create_street(crossroad, crossroad)
	
	
	
	
#	street.set_next(null)
#
#	var old_end = street.end	
#	street.set_end(crossroad)
#
#	var new_street = _create_street(crossroad, _create_intersection(end_point))
#	var previous = new_street.start.previous_street(new_street)
#	new_street.set_previous(previous)
#
#	street.set_next(crossroad.next_street(street))
#
#	var old_street2 = _create_street(crossroad, old_end)
#	old_street2.set_previous(crossroad.previous_street(old_street2))

	
	return crossroad	
	
func _split_street(street, split_point, end_point, end = null):
	pass
	

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



func _intersect_with_street(position):
	var near_intersection = _intersection_manager.is_near_intersection(position, 50)
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
