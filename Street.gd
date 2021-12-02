class_name Street
extends Buildable

# intersections
var start = null
var end  = null 

var left_district : District = null
var right_district : District = null

var midpoints = []

var rng = RandomNumberGenerator.new()

var outline = false
const WIDTH = 10
const MIN_LENGTH = 50

var norm = Vector2(0, 0)
var angle = null

var _next = []
var _previous = []

onready var _id = get_index() setget set_id, get_id  

func set_district(district: District, side: int) -> void:
	assert(side >= 0 and side <= 1)
	
	if side == District.Side.LEFT:
		left_district = district
	if side == District.Side.RIGHT:
		right_district = district
		
func get_district(side: int) -> District:
	assert(side >= 0 and side <= 1)
	
	if side == District.Side.LEFT:
		return left_district
	if side == District.Side.RIGHT:
		return right_district
		
	return null

func set_id(id):
	_id = id
	
func get_id():
	return _id
	
func get_ui_name():
	return "Street"
	
func _ready():
	normal_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
	
	
	
	polygon.resize(4)
	
	_next.resize(2)
	_previous.resize(2)
	
	rng.randomize()
	
	update()
	
	._ready()
	
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
	
func set_start(new_start):
	if start:
		start.remove_outgoing_street(self)
		
	global_position = new_start.position
	position = new_start.position
	
	start = new_start
	new_start.update()
	
	update()
	
func _update_geometry():
	# add random midpoints
	var length = end.position.distance_to(position)
	
	norm = (end.position - position).normalized()
	angle = norm.angle()
	
	
	midpoints.clear()
	
	var perp_vec = Vector2(-norm.y, norm.x)
	
	polygon = PoolVector2Array( [
		perp_vec * WIDTH,
		end.position - global_position + perp_vec * WIDTH,
		end.position - global_position - perp_vec * WIDTH,
		-perp_vec * WIDTH
	] )

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
	if not end:
		return Vector2(0, 0)
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
	return Geometry.get_closest_point_to_segment_2d(point, global_position, end.global_position)
	
	
func distance_to(point : Vector2) -> float:
	return perpendicular_vec_to_point(point).length()
	
func get_side_of_point(point: Vector2) -> int:
	var v1 = (global_position - end.global_position)
	var v2 = (point - global_position)
		
	return District.Side.LEFT if v1.cross(v2) > 0 else District.Side.RIGHT

func _draw(): 
	draw_colored_polygon(polygon, color)

#	if outline:
#		var p = polygon
#		p.append(p[0])
#		draw_polyline(p, Color.white, 4)
#
#
#	#var norm = street.norm
	var perp_vec = Vector2(-norm.y, norm.x)
#
#	var start = norm * length() / 2.0
#	var dir_vec = -perp_vec * 50
#
#	draw_line(start, start + dir_vec, Color.orange, 4)
	



	if start and end:

		var polygon = []
		#var perp_vec = Vector2(-norm.y, norm.x)
		polygon.append(perp_vec * WIDTH + norm * (length() - 30))
		polygon.append(end.position - global_position)
		polygon.append(-perp_vec * WIDTH + norm * (length() - 30))	

		var color = Color(0, 1.0, 0, 0.8)
		draw_polygon(polygon, [color, color, color])	

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
	
