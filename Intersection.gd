class_name Intersection
extends Node2D 

enum Direction {OUT, IN}
var _streets = []
var _streets_dir = []

var _cnt_incoming_streets = 0
var _cnt_outgoing_streets = 0

const MAX_FLOAT = 2147483647

const INTERSECTION_STREET_LENGTH = 80

var color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)

onready var _id = get_index() setget set_id, get_id  

# ==============================================================================

class MyCustomSorter:
	static func _angle(vec : Vector2):	
		var d = vec.angle()
		d = (d if d > 0 else (2*PI + d)) * 360 / (2*PI)
			
		if d == 360:
			return 0
			
		return d
		
	static func sort_ascending(a : Dictionary, b : Dictionary):
		var a_norm = a.street.norm if a.dir == Direction.OUT else (a.street.start.position - a.street.end.position).normalized()
		var b_norm = b.street.norm if b.dir == Direction.OUT else (b.street.start.position - b.street.end.position).normalized()
		
		if _angle(a_norm) < _angle(b_norm):
			return true
			
		return false
		
# ==============================================================================

func set_id(id):
	_id = id
	
func get_id():
	return _id
	
func save():
	var _street_ids = []
	for s in _streets: 
		_street_ids.append(s["street"].get_id())
		
	var save_dict = {
		"id": _id,
		"pos_x": position.x,
		"pos_y": position.y,
		"streets": _street_ids				
	}
	
	return save_dict

func update():
	_reorder()
	
	.update()

func remove_street(street):
	for s in _streets:
		if s["street"] == street:
			_streets.erase(s)
				
	_reorder()
	
func _angle(vec : Vector2):	
	var d = vec.angle()
	d = (d if d > 0 else (2*PI + d)) * 360 / (2*PI)
			
	return d	

static func _angle_in_360deg(vec : Vector2):	
		var d = vec.angle()
		d = (d if d > 0 else (2*PI + d)) * 360 / (2*PI)
		
		if d == 360:
			return 0
			
		return d	
		
func norm_of_street(street, dir: int) -> Vector2:
	return street.norm if dir == Direction.IN else (street.start.position - street.end.position).normalized()

func _get_index_for_new_street(street, dir) -> int:
	if _streets.empty():
		return 0
		
	for i in range(0, _streets.size()):
		var angle = _angle_in_360deg(norm_of_street(_streets[i].street, _streets[i].dir))		
		var new_angle = _angle_in_360deg(norm_of_street(street, dir))
		
		if new_angle < angle: 
			return i
			
	return _streets.size() - 1		
		
func _reorder():
	_streets.sort_custom(MyCustomSorter, "sort_ascending")

	for i in range(0, _streets.size()):
		var street = _streets[i].street
		
		var previous = _streets[i-1].street if i > 0 else _streets.back().street
		var next = _streets[i+1].street if i < _streets.size()-1 else _streets.front().street
		
		if _streets[i].dir == Direction.IN:
			street.set_next(null, District.Side.LEFT)
			street.set_next(null, District.Side.RIGHT)
						
			if previous.get_id() != street.get_id():
				street.set_next(previous, District.Side.RIGHT)
			
			if next.get_id() != street.get_id():
				street.set_next(next, District.Side.LEFT)
				
			street.update()
		else:
			street.set_previous(null, District.Side.LEFT)
			street.set_previous(null, District.Side.RIGHT)
			
			if next.get_id() != street.get_id():
				street.set_previous(next, District.Side.RIGHT)
			
			if previous.get_id() != street.get_id():
				street.set_previous(previous, District.Side.LEFT)
				
			street.update()

func add_outgoing_street(street): 
	if contains_street(street):
		return 
		
	_streets.push_back({ "dir": Direction.OUT, "street": street})
	_reorder()

	_cnt_outgoing_streets += 1	
			
func remove_outgoing_street(street):
	remove_street(street)
	_reorder()		
		
	_cnt_outgoing_streets -= 1
	
func contains_street(street) -> bool:
	for s in _streets:
		if s.street == street:
			return true
			
	return false
	
func add_incoming_street(street):
	if contains_street(street):
		return 
		
	_streets.push_back({ "dir": Direction.IN, "street": street})
	update()

	_cnt_incoming_streets += 1
	
func remove_incoming_street(street):
	remove_street(street)
	update()
	
	
	_cnt_incoming_streets -= 1
	


func has_incoming_streets():
	return _cnt_incoming_streets > 0

func has_outgoing_streets():
	return _cnt_outgoing_streets > 0
	
func _dir_of_street(street):
	for s in _streets:
		if s["street"] == street:
			return s["dir"]
		
	return null
	
func _angle_between_vecs(vec1, vec2):	
	var d = vec1.angle_to(vec2)
	
	if d < 0:
		d += 2*PI
		
	return d

func previous_street(street):
	var index = _streets.find(street)
	
	if index == 0:
		return _streets.back()
	
	return _streets[index - 1].street


func _find(street):
	for i in range(_streets.size()):
		if _streets[i].street == street:
			return i
	
	return -1

func next_street(street):
	var index = _find(street)
	
	assert(index != -1)
	
	if index == _streets.size() - 1:
		return _streets[0].street
		
	return _streets[index + 1].street

func get_index_of_street(street) -> int:
	var index = -1
	for i in range(_streets.size()):
		if _streets[i].street.get_id() == street.get_id():
			index = i
			
	return index

func get_norm_of_adjacent_streets(street) -> Array:
	var index = get_index_of_street(street)
	var p = index - 1 if index > 0 else _streets.size() - 1
	var n = index + 1 if index < _streets.size() - 1 else 0

	var p_norm = _streets[p].street.norm if _streets[p].dir == Direction.OUT else _streets[p].street.inverse_norm
	var n_norm = _streets[n].street.norm if _streets[n].dir == Direction.OUT else _streets[n].street.inverse_norm
	
	return [p_norm, n_norm]			
		
func get_angles_to_adjacent_streets(street) -> Array:
	if _streets.size() == 1:
		return [PI, PI]
		
	var index = get_index_of_street(street)			
	assert(index != -1)
	
	var norm = street.norm if _streets[index].dir == Direction.OUT else street.inverse_norm
	var adjacent_norms = get_norm_of_adjacent_streets(street)
	var p_angle = norm.angle_to(adjacent_norms[0]) 
	var n_angle = norm.angle_to(adjacent_norms[1])
	
	return [p_angle, abs(n_angle)]
	
	
func previous_angle_to_line(end):
	var dd = MAX_FLOAT
	var smallest = null
	
	var norm = (end.global_position - global_position).normalized()
	for s in _streets:		
		var d
		if s["dir"] == Direction.OUT:
			d = _angle_between_vecs(norm, s["street"].norm)
		else:
			d = _angle_between_vecs(norm, (s["street"].start.position - s["street"].end.position).normalized())
			
		if d > 0.0 and d < dd:
			dd = d
			smallest = s
			
	return dd	
	
func next_angle_to_line(end):
	var dd = -MAX_FLOAT
	var smallest = null
	
	var norm = (end.global_position - position).normalized()
	for s in _streets:		
		var d
		if s["dir"] == Direction.OUT:
			d = _angle_between_vecs(norm, s["street"].norm)
		else:
			d = _angle_between_vecs(norm, (s["street"].start.position - s["street"].end.position).normalized())
			
		if d > dd:
			dd = d
			smallest = s
			
	dd = 2 * PI - dd		
	return dd	

func _input(event):
	if event is InputEventMouseMotion:	
		var distance = event.global_position.distance_to(global_position)
		
		if distance < 50:
			var ids = []
			for i in _streets:
				ids.push_back(i.street.get_id())
	
func _draw(): 	
	var points = []
	var position = self.global_position
	var SIZE = 20
	
	var valid_streets = []
	for s in _streets:
		#if s.street.is_constructable():
			valid_streets.push_back(s)	
	
	var colors = []
	if valid_streets.empty():
		return
	
	if valid_streets.size() == 1:
		var norm = valid_streets[0].street.norm if valid_streets[0].dir == Direction.OUT else valid_streets[0].street.inverse_norm
		var perp = Vector2(-norm.y, norm.x)
		
		var offset = perp * 10
		var length = min(INTERSECTION_STREET_LENGTH, valid_streets[0].street.length)
		points.push_back(offset)	
		points.push_back(norm * length + offset)	
		points.push_back(norm * length - offset)	
		points.push_back(-offset)	
		
		colors.append_array([
			valid_streets[0].street.normal_color,
			valid_streets[0].street.normal_color,
			valid_streets[0].street.normal_color,
			valid_streets[0].street.normal_color
			])
	else:
		for i in range(valid_streets.size()):
			
			var _i = 0 if i == valid_streets.size() else i
			var _p = _i - 1 if _i > 0 else valid_streets.size() - 1
			
			var p_norm =  valid_streets[_p].street.norm if valid_streets[_p].dir == Direction.OUT else valid_streets[_p].street.inverse_norm
			var norm = valid_streets[_i].street.norm if valid_streets[_i].dir == Direction.OUT else valid_streets[_i].street.inverse_norm
			
			
			var p_length = min(INTERSECTION_STREET_LENGTH, valid_streets[_p].street.length)
			var length = min(INTERSECTION_STREET_LENGTH, valid_streets[_i].street.length)
		
			var p_perp = Vector2(-p_norm.y, p_norm.x)			
			var perp = Vector2(-norm.y, norm.x)
		
			var intersection = Geometry.line_intersects_line_2d(p_norm * 10 + p_perp * 10, p_norm, norm * 10 - perp * 10, norm)
			
			# special case if a street was splitted and previous and current street have the same 
			# normal. In this case we cannot use the intersection 
			var t = (p_norm.rotated(3.141593) - norm).abs()
			if is_equal_approx(t.x, 0.0) and is_equal_approx(t.y, 0.0):
				intersection = p_perp * 10
	
	
			if intersection and intersection.length() > INTERSECTION_STREET_LENGTH:
				intersection = intersection.normalized() * INTERSECTION_STREET_LENGTH
			
			points.push_back(p_norm * p_length + p_perp * 10)		
			points.push_back(intersection)
			points.push_back(norm * length - perp * 10)
			
			colors.push_back(valid_streets[_p].street.normal_color)
			colors.push_back(Color(42.0 / 255, 42.0 / 255, 43.0 / 255))
			colors.push_back(valid_streets[_i].street.normal_color)
			

	draw_polygon(points, colors)
	#draw_polyline(points, Color.black)
	#draw_colored_polygon(points, Color(42.0 / 255, 42.0 / 255, 43.0 / 255))
	
	draw_circle(Vector2(0, 0), 10, Color.orange)
