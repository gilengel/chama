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

var temp_building : Building = null
var temp_street : Street = null
var temp_district : District = null

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
	
func create_building(building : Building, district : District) -> Building:
	var new_building = building.duplicate()
	#new_building.position = ExtendedGeometry.centroid_polygon_2d(district.get_points())
	new_building.district = district
	new_building.add_to_group(BUILDING_GROUP)
	new_building.add_to_group($"../".PERSIST_GROUP)
	new_building.visible = true
	add_child(new_building)
	
	return new_building

# Called when the node enters the scene tree for the first time.
func _ready():
	entity_group = BUILDING_GROUP

	_gui_main_panel.connect("building_changed", self, "_change_temp_building")
	_gui_main_panel.connect("destroy", self, "_enable_destroy")
	
func _enable_destroy():
	destroy_enabled = true
	
func _change_temp_building(building : Buildable):
	destroy_enabled = false
	
	if building:
		enabled = true
		
		remove_child(temp_building)
		temp_building = building.duplicate()
		temp_building.visible = false
		add_child(temp_building)
	else:
		enabled = false
		
		remove_child(temp_building)


func _get_influenced_districts(district, max_recursion = 1, _result = [], iteration = 0):
	
	if iteration == max_recursion:
		_result.push_back(district)
		return _result
		
		
	for neighbour in district.neighbours:
		var exists = false
		print(_result)
		for d in _result:
			if d.get_id() == neighbour.get_id():
				exists = true
				
		if not exists:
			_result.push_back(neighbour)
		
		
	for neighbour in district.neighbours:
		_result.append_array(_get_influenced_districts(neighbour, max_recursion, _result, iteration + 1))
	return _result

func _input_build(event):
	if event is InputEventMouseMotion:
		temp_district = null
		
		for district in _district_manager.get_all():
			district.set_hovered(false)
			
		for district in _district_manager.get_all():
			if district.is_point_in_district(_mouse_world_position):
				
				var influenced_districts = _get_influenced_districts(district, 2)
			
				district.hover_color = Color.orangered
				district.set_hovered(true)
				temp_district = district
				
				for n in district.neighbours:
					n.hover_color = Color.orange
					n.set_hovered(true)
					
	if event.is_action_pressed("place_object") and temp_building.is_constructable() and temp_district:	
		create_building(temp_building, temp_district) 

func _input_destroy(event):
	if event is InputEventMouseMotion:		

		if temp_street:
			temp_street.set_hovered(false)
			
		temp_street =  _street_manager.is_point_on_street(_mouse_world_position)
		
		if temp_street:
			temp_street.set_hovered(true)
			
	if event.is_action_pressed("place_object") and temp_street:
		_district_manager.remove_district_via_street(temp_street, District.Side.LEFT)	
		_district_manager.remove_district_via_street(temp_street, District.Side.RIGHT)
		
		_street_manager.delete(temp_street)
		
		temp_street = null

func _input(event):
	._input(event)

	if enabled:
		_input_build(event)
		return
		
	if destroy_enabled:
		_input_destroy(event)
		

				
			
