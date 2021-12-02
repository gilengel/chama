class_name Intersection
extends Node2D 

enum Direction {OUT, IN}
var _streets = []
var _streets_dir = []

var _cnt_incoming_streets = 0
var _cnt_outgoing_streets = 0

const MAX_FLOAT = 2147483647

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

func remove_street(street):
	for s in _streets:
		if s["street"] == street:
			_streets.erase(s)
				
	if _streets.empty():
		queue_free()
		
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
	_streets.push_back({ "dir": Direction.OUT, "street": street})
	_reorder()

	_cnt_outgoing_streets += 1	
			
func remove_outgoing_street(street):
	remove_street(street)
	_reorder()		
	
	_cnt_outgoing_streets -= 1
	
	_check_for_deletion()
	
func add_incoming_street(street):
	_streets.push_back({ "dir": Direction.IN, "street": street})
	_reorder()

	_cnt_incoming_streets += 1
	
func remove_incoming_street(street):
	remove_street(street)
	_reorder()
	
	_cnt_incoming_streets -= 1
	
	_check_for_deletion()
	
func _check_for_deletion():
	if _cnt_incoming_streets == 0 and _cnt_outgoing_streets == 0:
		queue_free()

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
	
func previous_angle_to_line(end):
	var dd = MAX_FLOAT
	var smallest = null
	
	var norm = (end - position).normalized()
	for s in _streets:		
		var d
		if s["dir"] == Direction.OUT:
			d = _angle_between_vecs(norm, s["street"].norm)
		else:
			d = _angle_between_vecs(norm, (s["street"].start.position - s["street"].end.position).normalized())
			
		if d < dd:
			dd = d
			smallest = s
			
	return dd	
	
func next_angle_to_line(end):
	var dd = -MAX_FLOAT
	var smallest = null
	
	var norm = (end - position).normalized()
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
	
func _draw(): 	
	#draw_circle(Vector2(0, 0), 10, Color(0.2, 0.2, 0.2, 1))
	draw_rect(Rect2(Vector2(-10, -10), Vector2(20, 20)), color)
	
	var label = Label.new()
	var font = label.get_font("")

	#draw_string(font, Vector2(-4, 4), "%s" % get_id())
