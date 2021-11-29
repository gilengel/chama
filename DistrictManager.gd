class_name DistrictManager
extends Node2D


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var District = preload("res://District.gd")

onready var _street_manager = get_node("../StreetManager")

const DISTRICT_GROUP = "Districts"

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

func get_districts():
	return get_tree().get_nodes_in_group(DISTRICT_GROUP)

func get_district_by_id(id):
	for node in get_districts():
		if node.get_id() == id:
			return node
	
	return null

func load_district(data):
	var district = District.new()

	var pts = []
	for i in range(0, data.pts.size(), 2):
		pts.append(Vector2(data.pts[i], data.pts[i+1]))
	district.set_id(data.id)
	district.set_points(pts)
	district.side = data.side
	district.add_to_group(DISTRICT_GROUP)
	district.add_to_group($"../".PERSIST_GROUP)
	add_child(district)

func create_district_if_not_occupied(points):
	
	var intersects = false
	for district in get_districts():
		var intersection = Geometry.intersect_polygons_2d(points, district.get_points())
		
		if not intersection.empty():
			intersects = true
	
	if not intersects:
		var district = District.new()
		
		district.set_points(points)
		district.add_to_group(DISTRICT_GROUP)
		district.add_to_group($"../".PERSIST_GROUP)
		add_child(district)

func create_district(street, side):
	var district = District.new()
	
	var p = street.street_points(side)
	district.set_points(p)
	district.side = side
	district.street = street
	district.add_to_group(DISTRICT_GROUP)
	district.add_to_group($"../".PERSIST_GROUP)
	add_child(district)
	
	return district;
