class_name Street
extends Buildable

# intersections
var start = null setget set_start
var end  = null setget set_end

var left_district : District = null
var right_district : District = null

var midpoints = []

var rng = RandomNumberGenerator.new()

var outline = false
const WIDTH = 10
const MIN_LENGTH = 50

var norm = Vector2(0, 0)
var inverse_norm = Vector2(0,0)
var perp = Vector2(0, 0)
var inverse_perp = Vector2(0,0)
var angle = null
var length : float = 0.0

onready var _street_manager = get_node("../../StreetManager")
onready var _style_manager = get_node("../../StyleManager")

var _next = []
var _previous = []

enum PT { START_LEFT, END_LEFT, END_RIGHT, START_RIGHT }
enum Side { LEFT, RIGHT }

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
	
func get_ui_name():
	return "Street"
	
func _ready():
	rng.randomize()
	normal_color = Color(42.0 / 255, 42.0 / 255, 43.0 / 255)
	if _style_manager:
		normal_color = _style_manager.get_color(StyleManager.Colors.Street)	
	color = normal_color
	
	
	polygon.resize(4)
	
	_next.resize(2)
	_previous.resize(2)
	
	
	
	update()
	
	._ready()
	
func set_normal_color(new_normal_color: Color):
	normal_color = new_normal_color
	color = normal_color
	
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
		"d_l": "-1" if not is_instance_valid(left_district) else left_district.get_id(),
		"d_r": "-1" if not is_instance_valid(right_district) else right_district.get_id(),
	}
	
	return save_dict
	
func get_other_intersection(intersection : Intersection):
	return start if intersection == end else end

func set_previous(street, side : int):
	_previous[side] = street
	
	_update_geometry()

func get_previous(side : int):
	return _previous[side]
	
func set_next(street, side : int):
	_next[side] = street
	
	_update_geometry()

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
	
func _get_previous_point(side: int) -> Vector2:
	assert(side == 0 or side == 1)
	
	var p = _previous[side]	
	
	if p.end == start:
		return p.polygon[1 if side == District.Side.RIGHT else 2]
	
	
	return p.polygon[0 if side == District.Side.RIGHT else 3]
		
func update_points(indices, points):
	for i in range(indices.size()):
		polygon[indices[i]] = points[i]
		
	update()	

func _get_previous_indices(side):
	return [PT.END_LEFT, PT.END_RIGHT] if _previous[side].start != start else [PT.START_RIGHT, PT.START_LEFT]

func _get_next_indices(side):
	return [PT.START_LEFT, PT.START_RIGHT] if _next[side].end != end else [PT.END_RIGHT, PT.END_LEFT]
	
func street_points(distance = WIDTH, distance2 = 60):
	var s = start.global_position
	var e = end.global_position
	
	var length = e.distance_to(s)

	var p = []
#	var l = min(Intersection.INTERSECTION_STREET_LENGTH, length)
	var offset = perp * distance
#	p.append(norm * l - offset)
#	p.append(norm * (length - l) - offset)
#	p.append(norm * (length - l) + offset)
#	p.append(norm * l + offset)
	
	var multiple_streets_at_start = start._streets.size() > 2
	if multiple_streets_at_start:
		p.append(Vector2(0, 0))
	
	p.append(-offset)	
	p.append(norm * length - offset)
	
	if end._streets.size() > 2:
		p.append(norm * length)
		
	p.append(norm * length + offset)
	p.append(offset)
	
	
	
	
	if _next[District.Side.LEFT]:
		var l_next = _next[District.Side.LEFT]

		var n_offset = -l_next.perp * WIDTH if l_next.end != end else l_next.perp * WIDTH
		var intersection = Geometry.line_intersects_line_2d(p[0 if not multiple_streets_at_start else 1], norm, n_offset, l_next.norm)

		if intersection:
			p[1 if not multiple_streets_at_start else 2] = norm * length + intersection

	if _next[District.Side.RIGHT]:
		var l_next = _next[District.Side.RIGHT]

		var n_offset = l_next.perp * WIDTH if l_next.end != end else -l_next.perp * WIDTH
		var intersection = Geometry.line_intersects_line_2d(p[p.size() - 1], norm, n_offset, l_next.norm)

		if intersection:
			p[p.size() - 2] = norm * length + intersection			


	if _previous[District.Side.LEFT]:
		var l_previous = _previous[District.Side.LEFT]

		var p_offset = -l_previous.perp * WIDTH if l_previous.start != start else l_previous.perp * WIDTH
		var intersection = Geometry.line_intersects_line_2d(p[0 if not multiple_streets_at_start else 1], norm, p_offset, l_previous.norm)

		if intersection:
			p[0 if not multiple_streets_at_start else 1] = intersection

	if _previous[District.Side.RIGHT]:
		var l_previous = _previous[District.Side.RIGHT]

		var p_offset = l_previous.perp * WIDTH if l_previous.start != start else -l_previous.perp * WIDTH
		var intersection = Geometry.line_intersects_line_2d(p[p.size() - 1], norm, p_offset, l_previous.norm)

		if intersection:
			p[p.size() - 1] = intersection	
			
	return p
	
func _update_geometry():	
	# add random midpoints
	length = end.position.distance_to(position)
	
	norm = (end.position - position).normalized()
	inverse_norm = (position - end.position).normalized()
	angle = norm.angle()

	
	
	midpoints.clear()
	
	perp = Vector2(-norm.y, norm.x)
	inverse_perp = Vector2(-inverse_norm.y, inverse_norm.x)


	polygon = PoolVector2Array(street_points())

	update()

		
func set_end(new_end):
	if end:
		end.remove_incoming_street(self)
	
	end = new_end
	new_end.add_incoming_street(self)
	new_end.update()	

	
	
	start.add_outgoing_street(self)

	

	_update_geometry()
	
func set_end_position(new_end_pos):
	end.position = new_end_pos
	
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


const MIN_ANGLE = 0.785398 # 45°
const MAX_CONNECTIONS_PER_INTERSECTION = 4
const MAX_LENGTH = 2000

func is_constructable():
	var min_length = _violates_min_length()
	var max_length = _violates_max_length()
	var exceeds_min_angle = _exceeds_min_angle()
	
	return not min_length and not max_length and exceeds_min_angle

func _violates_min_length():
	return start.global_position.distance_to(end.global_position) < MIN_LENGTH

func _violates_max_length():
	return start.global_position.distance_to(end.global_position) > MAX_LENGTH

func _violates_max_streets_on_intersection():
	return start._streets.size() >= MAX_CONNECTIONS_PER_INTERSECTION

func _exceeds_min_angle():
	var angles = start.get_angles_to_adjacent_streets(self)
	return (
		angles[0] <= -MIN_ANGLE or angles[0] >= MIN_ANGLE and 
		angles[1] <= -MIN_ANGLE or angles[1] >= MIN_ANGLE
		)


#func _violates_intersecting_another_street():
#	if (_intersection_manager.is_near_intersection(start.global_position, SNAP_DISTANCE) or
#		_intersection_manager.is_near_intersection(end.global_position, SNAP_DISTANCE)):
#
#		return false
#
#	for street in _street_manager.get_all():
#		if Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, temp_street.points[0], temp_street.points[1]):
#			return true
#
#	return false

#func _draw():
#	#draw_polyline(polygon, _style_manager.get_color(StyleManager.Colors.Outline), 2)
#
#	var label = Label.new()
#	var font = label.get_font("")
##
##	draw_colored_polygon([
##		norm * length * 0.8,
##		norm * (length * 0.8-20) + perp * 10,
##		norm * (length * 0.8-20) - perp * 10,
##	], Color.limegreen)
##
#	var text = "%s -> %s %s %s %s" % [
#		get_id(),
#		"#" if not _previous[0] else _previous[0].get_id(),
#		"#" if not _previous[1] else _previous[1].get_id(),
#		"#" if not _next[0] else _next[0].get_id(),
#		"#" if not _next[1] else _next[1].get_id()		
#	]
#	draw_string(font, norm * length * 0.5, text, Color(1, 1, 1))
#	label.free()
