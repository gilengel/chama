class_name District
extends Buildable

var neighbours = []

var splits = 0

enum Side {LEFT, RIGHT}

const MIN_AREA_SIDE = 100

var rng = RandomNumberGenerator.new()

var houses: Array = []

onready var _style_manager = get_node("../../StyleManager")

class BinaryTreeNode:
	var parent : BinaryTreeNode = null
	var left : BinaryTreeNode = null
	var right : BinaryTreeNode = null
	var value = null
	
	func _init(new_parent : BinaryTreeNode):
		self.parent = new_parent

	

func _ready():	
	if _style_manager:
		normal_color = _style_manager.get_color(StyleManager.Colors.District)
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
