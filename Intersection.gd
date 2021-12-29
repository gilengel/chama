class_name Intersection
extends Polygon2D 

enum Direction {OUT, IN}
var _streets = []
var _streets_dir = []

var _cnt_incoming_streets = 0
var _cnt_outgoing_streets = 0

const MAX_FLOAT = 2147483647

const INTERSECTION_STREET_LENGTH = 80

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
	
	#update_geometry()
	
	.update()
	
func update_geometry():
	var pts = []
	for s in _streets:
		pts.push_back(s.street.polygon[0] if s.dir == Direction.OUT else s.street.polygon[1] - s.street.norm * s.street.length)

	polygon = PoolVector2Array(pts)
	
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

func previous(street):
	var index = index_of(street)
	
	if index == 0:
		return _streets.back()
	
	return _streets[index - 1]
	
func previous_street(street):
	return previous(street).street
	
func direction(street):
	var index = index_of(street)
	
	assert(index >= 0)
	
	return _streets[index].dir


func index_of(street):
	for i in range(_streets.size()):
		if _streets[i].street == street:
			return i
	
	return -1

func next(street):
	var index = index_of(street)
	
	assert(index != -1)
	
	if index == _streets.size() - 1:
		return _streets[0]
		
	return _streets[index + 1]	
	
func next_street(street):
	return next(street).street

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
	
	return [p_angle, n_angle]
	
	
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
