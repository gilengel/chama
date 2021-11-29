class_name Street
extends Node2D

# intersections
var start = null
var end  = null 

var _left_district = null setget set_left_district, get_left_district
var _right_district = null setget set_right_district, get_right_district

var District = preload("res://District.gd")

var midpoints = []

var rng = RandomNumberGenerator.new()

var color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
var outline = false
const WIDTH = 10
const MIN_LENGTH = 50

var norm = Vector2(0, 0)
var angle = null

var polygon = []

#var _previous setget set_previous, get_previous # Street
#var _next setget set_next, get_next # Street

var _next = []
var _previous = []

onready var _id = get_index() setget set_id, get_id  

func set_id(id):
	_id = id
	
func get_id():
	return _id
	
func _ready():
	_next.resize(2)
	_previous.resize(2)
	
	rng.randomize()
	
	update()
	
func save():
	var save_dict = {
		"id": _id,
		"start": start.get_id(),
		"end": end.get_id(),
		"p_l": "null" if not _previous[District.Side.LEFT] else _previous[District.Side.LEFT].get_id(),
		"p_r": "null" if not _previous[District.Side.RIGHT] else _previous[District.Side.RIGHT].get_id(),
		"n_l": "null" if not _next[District.Side.LEFT] else _next[District.Side.LEFT].get_id(),
		"n_r": "null" if not _next[District.Side.RIGHT] else _next[District.Side.RIGHT].get_id(),				
	}
	
	return save_dict
	
func get_other_intersection(intersection : Intersection):
	return start if intersection == end else end

func set_previous(street, side : int):
	_previous[side] = street

func get_previous(side : int):
	return _previous[side]
	
func set_next(street, side : int):
	_next[side] = street

func get_next(side : int):
	return _next[side]
	
func get_next_from_intersection(side : int, point : Intersection):
	if point == end:
		return get_next(side)
	else:
		return get_previous(side)

func get_previous_from_intersection(side : int, point : Intersection):
	if point == start:
		return get_next(side)
	else:
		return get_previous(side)	
	
func set_left_district(district):
	_left_district = district

func get_left_district():
	return _left_district
	
func set_right_district(district):
	_right_district = district

func get_right_district():
	return _right_district
	
func set_start(new_start):
	if start:
#		if _left_district:
#			_left_district.intersections.erase(start)
#
#		if _right_district:
#			_right_district.intersections.erase(start)
		
		start.remove_outgoing_street(self)
		
	global_position = new_start.position
	position = new_start.position
	
	start = new_start
	new_start.update()
	
	update()
	
func get_district(side):
	if side == District.Side.LEFT:
		return _left_district
	else:
		return _right_district
	
func _update_geometry():
	# add random midpoints
	var length = end.position.distance_to(position)
	
	norm = (end.position - position).normalized()
	angle = norm.angle()
	
	
	midpoints.clear()
	
	var perp_vec = Vector2(-norm.y, norm.x)
	polygon.clear()
	polygon.append(perp_vec * WIDTH)
	polygon.append(end.position - global_position + perp_vec * WIDTH)
	polygon.append(end.position - global_position - perp_vec * WIDTH)
	polygon.append(-perp_vec * WIDTH)	
	
	update()

		
func set_end(new_end):
	if end:
		end.remove_incoming_street(self)
	
	end = new_end
	new_end.add_incoming_street(self)
	new_end.update()
	
	
	

	_update_geometry()
	
	start.add_outgoing_street(self)
	
	#for i in range(floor(length / 25) - 1):
	#	midpoints.append(start + norm * 25 * (i+1) + Vector2(-norm.y, norm.x) * rng.randf_range(-5, 5))
		
	update()
	
func set_end_position(new_end_pos):
	end.position = new_end_pos
	
	_update_geometry()
	update()
	
func length():
	return vec().length()
	
func vec():
	return (end.position - position)
	
func global_polygon():
	var points = []
	
	for point in polygon:
		points.append(point + position)

	return points

func intersection(anotherStreet):
	var intersection = Geometry.segment_intersects_segment_2d(position, end.position, anotherStreet.position, anotherStreet.end.position)
	
	#if intersection and Geometry.is_point_in_polygon(intersection, global_polygon()):
	#	return intersection
	return intersection	
	#return null



func _calc_intersections(pts, other_pts):
	var inner_intersection = Geometry.line_intersects_line_2d(other_pts[3], other_pts[0] - other_pts[3], pts[3], (pts[0] - pts[3]))
	var outer_intersection = Geometry.line_intersects_line_2d(other_pts[1], other_pts[2] - other_pts[1], pts[2], (pts[2] - pts[1]))
	
	# corner case if lines are perfectly horizontal/vertical to each other
	if not inner_intersection:
		inner_intersection = Vector2((other_pts[3] + other_pts[0] + pts[3] + pts[0]) / 4)
		outer_intersection = Vector2((other_pts[1] + other_pts[2] + pts[2] + pts[1]) / 4)	
		
	return [inner_intersection, outer_intersection]
	
func perpendicular_vec_to_point(point : Vector2) -> Vector2:
#	var x2 = end.global_position.x
#	var x1 = global_position.x
#	var x0 = point.x
#
#	var y2 = end.global_position.y
#	var y1 = global_position.y
#	var y0 = point.y
#
#	var k = ((y1-y0) * (x2-x0) - (x1-x0) * (y2-y0)) / (pow(y1-y0, 2) + pow(x1-x0, 2))
#
#	return Vector2(x2 - k * (y1-y0), y2  k * (x1-x0))

	return Geometry.get_closest_point_to_segment_2d(point, global_position, end.global_position)
	
	
func distance_to(point : Vector2) -> float:
#	var x2 = end.global_position.x
#	var x1 = global_position.x
#	var x0 = point.x
#
#	var y2 = end.global_position.y
#	var y1 = global_position.y
#	var y0 = point.y
#
#	return abs((x2-x1)*(y1-y0) - (x1-x0)*(y2-y1)) / sqrt(pow(x2-x1, 2)+pow(y2-y1, 2))

#	var d1 = global_position.distance_to(point)
#	var d2 = end.global_position.distance_to(point)
#	return d1 if d1 < d2 else d2

	return perpendicular_vec_to_point(point).length()

func street_points(side, distance = 10, distance2 = 60):
	var pts = _foo(side, distance, distance2)
#	var other_pts = []
#
#	var other_indices = []
#	var other_district = null
#
#	# in case the streets starts in a connection
#	if _previous:		
#		if start == _previous.start:
#			var other_side = District.Side.LEFT if side == District.Side.RIGHT else District.Side.RIGHT	
#
#			other_pts = _previous._foo(other_side, distance, distance2)
#
#			other_indices = [0, 1]
#			other_district = _previous.get_district(other_side)
#
#		else:
#			other_pts = _previous._foo(side, distance, distance2)
#			other_indices = [3, 2]
#			other_district = _previous.get_district(side)
#
#
#		var intersections = _calc_intersections(pts, other_pts)
#
#		pts[0] = intersections[0]
#		pts[1] = intersections[1]			
#
#		other_district.update_points(other_indices, intersections)
#
#
#	# in case the streets end in a connection
#	if _next:
#		if end == _next.end:			
#			var other_side = District.Side.LEFT if side == District.Side.RIGHT else District.Side.RIGHT	
#
#			other_pts = _next._foo(other_side, distance, distance2)
#			other_indices = [3, 2]
#			other_district = _next.get_district(other_side)
#		else:
#			other_pts = _next._foo(side, distance, distance2)
#			other_indices = [0, 1]
#			other_district = _next.get_district(side)
#
#
#		var intersections = _calc_intersections(pts, other_pts)				
#
#		pts[3] = intersections[0]
#		pts[2] = intersections[1]
#
#
#		#if other_district:
#		#	other_district.update_points(other_indices, intersections)	
		
	return pts
	
func _foo(side, distance, distance2):
	var perp_vec = Vector2(-norm.y, norm.x)
	
	var s = start.position
	var e = end.position
		
	var p = []
	

	if side == District.Side.LEFT:
		p.append(s - perp_vec * distance)
		p.append(s - perp_vec * distance2)
		
		p.append(e - perp_vec * distance2)
		p.append(e - perp_vec * distance)		
		
	else:
		p.append(s + perp_vec * distance)
		p.append(s + perp_vec * distance2)
		
		p.append(e + perp_vec * distance2)
		p.append(e + perp_vec * distance)
		
	return p
	
func get_side_of_point(point: Vector2) -> int:
	var v1 = (global_position - end.global_position)
	var v2 = (point - global_position)
		
	return District.Side.LEFT if v1.cross(v2) > 0 else District.Side.RIGHT

func print():
	var a = get_previous(District.Side.LEFT)
	var b = get_previous(District.Side.RIGHT)
	var c = get_next(District.Side.LEFT)
	var d = get_next(District.Side.RIGHT)	
	
	var text = "%s -> pl[%s] pr[%s] nl[%s] nr[%s]" % [
		get_id(), 
		a.get_id() if a else "#", 
		b.get_id() if b else "#", 
		c.get_id() if c else "#", 
		d.get_id() if d else "#", 
	]
	print(text)

func _draw(): 
	draw_colored_polygon(polygon, color)
	
	if outline:
		var p = polygon
		p.append(p[0])
		draw_polyline(p, Color.white, 4)
	
	if start and end:

		var polygon = []
		var perp_vec = Vector2(-norm.y, norm.x)
		polygon.append(perp_vec * WIDTH + norm * (length() - 30))
		polygon.append(end.position - global_position)
		polygon.append(-perp_vec * WIDTH + norm * (length() - 30))	

		var color = Color(0, 1.0, 0, 0.8)
		draw_polygon(polygon, [color, color, color])	
#


		var a = get_previous(District.Side.LEFT)
		var b = get_previous(District.Side.RIGHT)
		var c = get_next(District.Side.LEFT)
		var d = get_next(District.Side.RIGHT)
				
		var label = Label.new()
		var font = label.get_font("")

		var text = "%s -> %s,%s,%s,%s" % [
			get_id(), 
			a.get_id() if a else "#", 
			b.get_id() if b else "#", 
			c.get_id() if c else "#", 
			d.get_id() if d else "#", 
		]
		
		var v = (end.position - global_position).normalized() * (end.position - global_position).length() / 2.0 - Vector2(40, 0)
		draw_string(font, v + Vector2(0,7), text, Color.white)		

	
