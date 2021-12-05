class_name District
extends Buildable

var neighbours = []
# Declare member variables here. Examples:
var _geometry = []
var _triangles = []

var splits = 1

enum Side {LEFT, RIGHT}

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


func _line_segment(v1: Vector2, v2: Vector2, centroid: Vector2) -> Dictionary:
		var vec = (v2 - v1)
		var norm = vec.normalized()
		var anorm = (v1 - v2).normalized()
		var perp = Vector2(-norm.y, norm.x)	
				
		return { "s": v1, "e": v2, "n": norm, "an": anorm, "p": perp, "l": vec.length()}

#func _before(i: int):
#	if i == 0:
#		return _line_segment(_geometry.back(), _geometry.front())
#
#	return _line_segment(_geometry[i-1], _geometry[i])
	
#func _next(i: int):
#	if i+1 == _geometry.size() - 1:
#		return _line_segment(_geometry.front(), _geometry.back())	
#
#	return _line_segment(_geometry[i+1], _geometry[i+2])
	
const MIN_AREA = pow(100, 2)

func generate_houses(polygon, max_splits):
	return _longest_side_starting_index(polygon, splits)
	
func _longest_side_starting_index(polygon, max_splits = 0, splits = 0, color = Color.black) -> Array:
	var polygons = []
	var values = []
		
	var centroid = ExtendedGeometry.centroid_polygon_2d(polygon)
	
	#if ExtendedGeometry.area_polygon_2d(polygon) < MIN_AREA or 
	if splits > max_splits :
		polygons.append({ "p": polygon, "c": color})
		return polygons
	
	var polygon_size = polygon.size()
	for i in range(polygon_size):
		values.append(_line_segment(polygon[i], polygon[(i+1) % polygon_size], centroid))
	values.append(_line_segment(polygon[polygon_size-1], polygon[0], centroid))
		
	var longest_side = values[0].l
	var longest_index = 0
	
	for i in range(1, values.size()):
		if values[i].l > longest_side:
			longest_side = values[i].l
			longest_index = i
	


	var midpoint = values[longest_index].s + values[longest_index].n * values[longest_index].l / 2.0
	var endpoint = midpoint + values[longest_index].p * 60000

	var intersections = []
	for i in range(0, values.size()):
		if i == longest_index:
			continue
		
		var intersection = Geometry.segment_intersects_segment_2d(midpoint, endpoint, values[i].s, values[i].e)
		
		if intersection:
			intersections.append({ "index": i, "point": intersection })


	var p1 = []
	p1.append(midpoint)
	var start = longest_index + 1
	var end = intersections[0].index
		
	var index = start - 1
	while index != end:
		index = (index + 1) % values.size()		
	
		p1.append(values[index].s)
		
	p1.append(intersections[0].point)
	p1.append(midpoint)

	start = intersections[0].index + 1
	end = longest_index
	
	print(p1)
	
	polygons.append_array(_longest_side_starting_index(p1, max_splits, splits + 1, Color(0, 1, 0, float(splits) / max_splits)))
	
	var p2 = []
	p2.append(intersections[0].point)
	
	index = start - 1
	while index != end:
		index = (index + 1) % values.size()
		
		p2.append(values[index].s)

	p2.append(midpoint)
	p2.append(intersections[0].point)
	
	polygons.append_array(_longest_side_starting_index(p2, max_splits, splits + 1, Color(1, 0, 0, float(splits) / max_splits)))
			 
	return polygons

const ANGLE_OFFSET = 0.5
				

func _draw(): 
	
	print("====")
	for s in generate_houses(ExtendedGeometry.order_polygon_2d_clockwise(_geometry), 3):
		#draw_colored_polygon(s.p, s.c)
		draw_polyline(s.p, Color.black, 3)
		
	if _geometry:

		var label = Label.new()
		var font = label.get_font("")

		var center = Vector2(0, 0)
		for g in _geometry:
			center += g
		center /= _geometry.size()

		draw_string(font, center, "%s n=%s" % [get_id(), neighbours.size()])
