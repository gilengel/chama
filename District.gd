class_name District
extends Buildable

var neighbours = []
# Declare member variables here. Examples:
var _geometry = []
var _triangles = []

enum Side {LEFT, RIGHT}

var rng = RandomNumberGenerator.new()

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

func _draw(): 
	
	for i in range(0, _triangles.size(), 3):
		var poly = [_geometry[_triangles[i]], _geometry[_triangles[i+1]], _geometry[_triangles[i+2]]]
		draw_polygon(poly,[color, color, color])

		
	if _geometry:

		var label = Label.new()
		var font = label.get_font("")

		var center = Vector2(0, 0)
		for g in _geometry:
			center += g
		center /= _geometry.size()

		draw_string(font, center, "%s n=%s" % [get_id(), neighbours.size()])
#
#		var length = _geometry[0].distance_to(_geometry[3])		
#		var norm = (_geometry[3] - _geometry[0]).normalized()
#		var perp_vec 
#
#		if side == Side.LEFT:
#			perp_vec = Vector2(-norm.y, norm.x)
#		else:
#			perp_vec = Vector2(-(_geometry[0] - _geometry[3]).y, (_geometry[0] - _geometry[3]).x).normalized()	
		
		
		
#		for i in range(0, length - fmod(length, 40), 40):
#
#			var depth = rng.randf_range(30, 50)
#			var gap = rng.randf_range(0, 10)
#			draw_colored_polygon([
#				_geometry[0] + norm * (i + gap),
#				_geometry[0] + norm * (i + gap) - perp_vec * depth,
#				_geometry[0] + norm * (i + 40) - perp_vec * depth,
#				_geometry[0] + norm * (i + 40),
#			], Color.black)
			
			#draw_rect(Rect2(, Vector2(40, 20)), Color.black, false, 5)

	
