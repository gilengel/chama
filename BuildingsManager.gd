extends Node

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _district_manager : DistrictManager = get_node("../DistrictManager")
onready var _street_manager : StreetManager = get_node("../StreetManager")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/MainPanel")

# ==============================================================================

var Building = preload("res://Building.gd")
var District = preload("res://District.gd")

# ==============================================================================

# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var temp_building : Building = null

var enabled = false

# Called when the node enters the scene tree for the first time.
func _ready():
	temp_building = Marketplace.new()
	add_child(temp_building)
	
	_gui_main_panel.connect("building_changed", self, "_change_temp_building")
	
func _change_temp_building(building : Buildable):
	if building:
		enabled = true
		
		remove_child(temp_building)
		temp_building = building.duplicate()
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
			
			#print("%s -> %s" % [street.get_id(), next.get_id()])
			
			points.append(street.start.position)
		else:
			next = street.get_previous(side)
			
			points.append(street.end.position)
			
		if next and (street.end == next.end or street.start == next.start):	
			forward = !forward
			
			side = District.Side.LEFT if side == District.Side.RIGHT else District.Side.RIGHT
				

		street = next
		
	return { "enclosed": next == start, "streets": streets, "points": points }
		
func _input(event):
	if not enabled:
		return 
		
	if event is InputEventMouseMotion:
		var s = _street_manager.get_closest_streets_to(event.global_position)
		var left_enclosed = _enclosed(s.street, s.side)
		if left_enclosed.enclosed:
			temp_building.polygon = left_enclosed.points
			
			if temp_building.area() < temp_building.min_area():
				temp_building.color = Color.orangered
			else:
				temp_building.color = Color.white

			
				
				


func _on_marketplace_toggled(button_pressed):
	print(":)")


func _on_build_marketplace_toggled():
	pass # Replace with function body.
	
