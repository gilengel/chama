class_name District
extends Buildable

var neighbours = []
# Declare member variables here. Examples:
var _geometry = []
var _triangles = []

var min_house_area: float = 0

var splits = 1

enum Side {LEFT, RIGHT}

const MIN_AREA_SIDE = 100

var rng = RandomNumberGenerator.new()

class BinaryTreeNode:
	var parent : BinaryTreeNode = null
	var left : BinaryTreeNode = null
	var right : BinaryTreeNode = null
	var value = null
	
	func _init(new_parent : BinaryTreeNode):
		self.parent = new_parent

	

func _ready():
	rng.randomize()
	
	if min_house_area == 0:
		min_house_area = pow(200, 2) #pow(MIN_AREA_SIDE * (1 + rng.randf_range(-0.15, 0.15)), 2)
	
	
	normal_color = Color(rng.randf(), rng.randf(), rng.randf(), 0.3)
	._ready()

func _save_neighbour_ids():
	var ids = []
	for i in neighbours:
		ids.append(i.get_id())
	
	return ids

func save():
	var pts = []
	for pt in _geometry:
		pts.append(pt.x)
		pts.append(pt.y)

	var save_dict = {
		"id": _id,
		"pos_x": position.x,
		"pos_y": position.y,
		"pts": pts,
		"neighbours": _save_neighbour_ids()
	}

	return save_dict
	
func get_points():
	return _geometry
	
func set_points(points):
	_geometry = points
			
	_triangles = Geometry.triangulate_polygon(_geometry)
	update()

func update_points(indices, points):
	for i in range(indices.size()):
		_geometry[indices[i]] = points[i]
		
	update()

func is_point_in_district(point):
	return Geometry.is_point_in_polygon(point, _geometry)


func _line_segment(v1: Vector2, v2: Vector2, is_street1: bool, is_street2: bool) -> Dictionary:
		var vec = (v2 - v1)
		var norm = vec.normalized()
		var perp = Vector2(-norm.y, norm.x)	
			
		return { "s": v1, "e": v2, "n": norm, "p": perp, "l": vec.length(), "street": is_street1 or is_street2}

func generate_houses(polygon, max_splits):
	var p = []
	for i in range(polygon.size()):
		p.push_back(true)
	
	var _p = polygon
	_p.push_back(_p[0])
	return _longest_side_starting_index(_p, p)
	
func _get_split_point(segment: Dictionary) -> Vector2:
	return segment.s + segment.n * segment.l * 0.5 # rng.randf_range(0.3, 0.7)
	
func _get_longest_side(values: Array, is_street: Array) -> Dictionary:
	assert(values.size() == is_street.size())
	
	# first filter side that are connected to a street because they
	# will be prioritized
	var values_with_streets = []
	for i in range(0, values.size()):
		if is_street[i]:
			values_with_streets.append(values[i])
		
	var _values = values if values_with_streets.empty() else values_with_streets
	
	var longest_side = _values[0].l
	var longest_index = 0	
	for i in range(1, _values.size()):
		if _values[i].l > longest_side:
			longest_side = _values[i].l
			longest_index = i
	
	return longest_index		

func _calculate_intersections(longest_index: int, midpoint: Vector2, values: Array) -> Array:
	var endpoint = midpoint + values[longest_index].p * 60000

	var intersections = []
	for i in range(0, values.size()):
		if i == longest_index:
			continue
		
		var intersection = Geometry.segment_intersects_segment_2d(midpoint, endpoint, values[i].s, values[i].e)
		
		if intersection:
			intersections.append({ "index": i, "point": intersection })
			
	return intersections
					
func _longest_side_starting_index(polygon: Array, is_street: Array, _splits = 0) -> Array:
	assert(is_street.size() == polygon.size() - 1)
	
	var polygons = []
	var values = []
	
	if ExtendedGeometry.area_polygon_2d(polygon) < min_house_area or _splits == splits:
	#if :
		polygons.append({ "p": polygon, "s": is_street })
		
		return polygons
	
	
	assert(polygon[0] == polygon[polygon.size()-1])
	polygon.pop_back()
	
	var polygon_size = polygon.size()
	
	for i in range(polygon_size):
		values.append(_line_segment(polygon[i], polygon[(i+1) % polygon_size], is_street[i], is_street[(i+1) % polygon_size]))
		
	var longest_index = _get_longest_side(values, is_street)
	var midpoint = _get_split_point(values[longest_index])
	var intersections = _calculate_intersections(longest_index, midpoint, values)

	var polygon1 = []
	polygon1.append(midpoint)
	var is_street1 = [is_street[longest_index]]
	
	var start = longest_index + 1
	var end = intersections[0].index
		
	var index = start - 1
	while index != end:
		index = (index + 1) % values.size()		
	
		polygon1.append(values[index].s)
		is_street1.append(is_street[index])
		
	polygon1.append(intersections[0].point)
	polygon1.append(midpoint)
	is_street1.append(false)
	
	polygons.append_array(_longest_side_starting_index(polygon1, is_street1, _splits + 1))

	start = intersections[0].index
	end = longest_index
	
	var polygon2 = []	
	polygon2.append(intersections[0].point)
	var is_street2 = [is_street[intersections[0].index]]
	index = start
	
	while index != end:
		index = (index + 1) % values.size()
				
		polygon2.append(values[index].s)
		is_street2.append(is_street[index])

	polygon2.append(midpoint)
	is_street2.append(false)
	polygon2.append(intersections[0].point)
		
	polygons.append_array(_longest_side_starting_index(polygon2, is_street2, _splits + 1))
	
	return polygons
				

func _draw(): 
	var label = Label.new()
	var font = label.get_font("")
			
	for s in generate_houses(ExtendedGeometry.order_polygon_2d_clockwise(_geometry), 3):
		
		for i in range(s.p.size()-1):
			draw_line(s.p[i], s.p[i+1], Color.black if s.s[i] else Color.red, 8)

		draw_line(s.p[s.p.size()-1], s.p[0], Color.black if s.s[s.p.size()-2] and s.s[0] else Color.red, 8)
		
#		draw_polyline(s.p, Color.black, 3)
		
	if _geometry:



		var center = Vector2(0, 0)
		for g in _geometry:
			center += g
		center /= _geometry.size()

		draw_string(font, center, "%s n=%s" % [get_id(), neighbours.size()])
