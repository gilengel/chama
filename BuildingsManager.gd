extends Node

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _district_manager : DistrictManager = get_node("../DistrictManager")
onready var _street_manager : StreetManager = get_node("../StreetManager")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/HBoxContainer/MainPanel")

const BUILDING_GROUP = "Buildings"

# ==============================================================================

var Building = preload("res://Building.gd")
var District = preload("res://District.gd")

# ==============================================================================

# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var temp_building : Building = null
var temp_street : Street = null

var enabled = false
var destroy_enabled = false

func create_building(building : Building, geometry: PoolVector2Array) -> Building:
	var new_building = building.duplicate()
	#new_building.position = new_building.centroid()
	new_building.add_to_group(BUILDING_GROUP)
	new_building.add_to_group($"../".PERSIST_GROUP)
	add_child(new_building)
	
	return new_building

# Called when the node enters the scene tree for the first time.
func _ready():
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


func _enclosed(start: Street, side : int):
	var next = start.get_next(side)
	var street = start		
	var forward = true
	
	var streets = []	
	var points = []
	var i = 0
	while next != start and next:
		streets.append(street)
		
		if forward:
			next = street.get_next(side)
			
			points.append(street.start.position)
		else:
			next = street.get_previous(side)
			
			points.append(street.end.position)
			
		if next and (street.end == next.end or street.start == next.start):	
			forward = !forward
			
			side = District.Side.LEFT if side == District.Side.RIGHT else District.Side.RIGHT
				

		street = next
		
	return { "enclosed": next == start, "streets": streets, "points": points }

func _input_build(event):
	if event is InputEventMouseMotion:
		var s = _street_manager.get_closest_streets_to(event.global_position)
		var left_enclosed = _enclosed(s.street, s.side)
		
		temp_building.visible = false
		if left_enclosed.enclosed:
			temp_building.polygon = left_enclosed.points
			temp_building.color = Color(1, 1, 1, 0.3)
			temp_building.visible = true
			
			
			if not temp_building.is_constructable():
				temp_building.color = Color.orange
			else:
				temp_building.color = Color.white

	if event.is_action_pressed("place_object") and temp_building.is_constructable():		
		create_building(temp_building, temp_building.polygon)

func _input_destroy(event):
	if event is InputEventMouseMotion:		
		if temp_street:
			temp_street.set_hovered(false)
			
		temp_street =  _street_manager.is_point_on_street(event.global_position)
		
		if temp_street:
			temp_street.set_hovered(true)
			
	if event.is_action_pressed("place_object") and temp_street:
		_street_manager.remove(temp_street)
		temp_street = null

func _input(event):
	if enabled:
		_input_build(event)
		return
		
	if destroy_enabled:
		_input_destroy(event)
		

				
			
