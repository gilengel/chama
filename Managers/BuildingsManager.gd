class_name BuildingsManager
extends EntityManager

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _district_manager : DistrictManager = get_node("../DistrictManager")
onready var _street_manager : StreetManager = get_node("../StreetManager")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/HBoxContainer/MainPanel")

const BUILDING_GROUP = "Buildings"

# ==============================================================================

#var Building = preload("res://Building.gd")
#var District = preload("res://District.gd")

# ==============================================================================



var enabled = false
var destroy_enabled = false

func preload_entity(data):
	var new_building : Building = null
	match data.type:
		"Church":
			new_building = Church.new()
			
	new_building.add_to_group(BUILDING_GROUP)
	new_building.add_to_group($"../".PERSIST_GROUP)
	new_building.visible = true
	add_child(new_building)
	
	new_building.set_id(data.id)

func load_entity(data):
	var building = get_by_id(data.id)
	var district = _district_manager.get_by_id(data.district)
	assert(district)
	
	building.position = ExtendedGeometry.centroid_polygon_2d(district.get_points())
	building.district = district
	

# Called when the node enters the scene tree for the first time.
func _ready():
	entity_group = BUILDING_GROUP

	_gui_main_panel.connect("building_changed", self, "_change_temp_building")
	_gui_main_panel.connect("destroy_mode_changed", self, "_enable_destroy")
	
func _enable_destroy(value):
	destroy_enabled = value
	
func create(type = null):
	assert(type != null)
	assert($Buildings.has_node(type))
	
	var building = $Buildings.get_node(type).duplicate()
	building.add_to_group(BUILDING_GROUP)
	building.add_to_group($"../".PERSIST_GROUP)
	add_child(building)
	
	return building

func is_point_on_building(point: Vector2):
	for building in get_all():
		if Geometry.is_point_in_polygon(point, building.shape()):
			return building
			
	return null
#func _input_destroy(event):
#	if event is InputEventMouseMotion:		
#
#		if temp_street:
#			temp_street.set_hovered(false)
#
#		temp_street =  _street_manager.is_point_on_street(_mouse_world_position)
#
#		if temp_street:
#			temp_street.set_hovered(true)
#
#	if event.is_action_pressed("place_object") and temp_street:
#		_street_manager.delete(temp_street)
#
#		temp_street = null
#
#func _input(event):
#	._input(event)
#
#	if enabled:
#		_input_build(event)
#		return
#
#	if destroy_enabled:
#		_input_destroy(event)
		

				
			
