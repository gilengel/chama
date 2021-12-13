class_name District
extends Buildable

var neighbours = []

var splits = 0

enum Side {LEFT, RIGHT}

const MIN_AREA_SIDE = 100

var rng = RandomNumberGenerator.new()

var houses: Array = []

class BinaryTreeNode:
	var parent : BinaryTreeNode = null
	var left : BinaryTreeNode = null
	var right : BinaryTreeNode = null
	var value = null
	
	func _init(new_parent : BinaryTreeNode):
		self.parent = new_parent

	

func _ready():	
	normal_color = Color(rng.randf(), rng.randf(), rng.randf(), 0.3)
	._ready()

func _save_neighbour_ids():
	var ids = []
	for i in neighbours:
		ids.append(i.get_id())
	
	return ids

func save():
	var pts = []
	for pt in polygon:
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
	return polygon
	
func set_points(points):
	polygon = points
	
	update()

func is_point_in_district(point):
	return Geometry.is_point_in_polygon(point, polygon)
	
func _draw():
	var center = ExtendedGeometry.average_centroid_polygon_2d(polygon)
	
	draw_circle(center, 20, Color.orange)
	
	center = ExtendedGeometry.centroid_polygon_2d(polygon)
	draw_circle(center, 20, Color.palevioletred)
