extends Node2D

# ==============================================================================

export var PERSIST_GROUP = "Persist"

# ==============================================================================

onready var _street_manager = get_node("StreetManager")
onready var _district_manager = get_node("DistrictManager")
onready var _intersection_manager = get_node("IntersectionManager")

onready var Building = get_node("Building")

# ==============================================================================

enum BUILD_MODE {NONE, STREET, BUILDING}

# ==============================================================================

var mode = BUILD_MODE.STREET

# ==============================================================================

func _ready():
	VisualServer.set_default_clear_color(Color(90.0 / 255, 148.0 / 255, 112.0 / 255 , 1.0))
	
	
	var mx = get_viewport().size.x / 2
	var my = get_viewport().size.y / 2



	# default street
	_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx+150, my-150))
	_street_manager.create_street(Vector2(mx+150, my-150), Vector2(mx+150, my+150))
	_street_manager.create_street(Vector2(mx+150, my+150), Vector2(mx-150, my+150))
	_street_manager.create_street(Vector2(mx-150, my+150), Vector2(mx-150, my-150))

	_street_manager.create_street(Vector2(mx+150, my-150), Vector2(mx+300, my-150))
	_street_manager.create_street(Vector2(mx+300, my+150), Vector2(mx+300, my-150))
	_street_manager.create_street(Vector2(mx+150, my+150), Vector2(mx+300, my+150))

	_street_manager.create_street(Vector2(mx+300, my-150), Vector2(mx+500, my-150))
	_street_manager.create_street(Vector2(mx+300, my+150), Vector2(mx+500, my+150))

	_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx-300, my-150))
	_street_manager.create_street(Vector2(mx-150, my+150), Vector2(mx-300, my+150))
	_street_manager.create_street(Vector2(mx-300, my-150), Vector2(mx-300, my+150))

func _save():
	var save_game = File.new()
	save_game.open("user://savegame.json", File.WRITE)

	var streets = []
	for street in get_tree().get_nodes_in_group(_street_manager.STREET_GROUP):
		streets.append(street.call("save"))
	
#	var districts = []
#	for district in get_tree().get_nodes_in_group(_district_manager.DISTRICT_GROUP):
#		districts.append(district.call("save"))

	var intersections = []
	for intersection in get_tree().get_nodes_in_group(_intersection_manager.INTERSECTION_GROUP):
		intersections.append(intersection.call("save"))
	
	var save_dict = {
		"streets": streets,
		#"districts": districts,
		"intersections": intersections			
	}
		
	save_game.store_line(to_json(save_dict))		
	save_game.close()
	

func _load():
	var save_game = File.new()
	if not save_game.file_exists("user://savegame.json"):
		return # Error! We don't have a save to load.

	# We need to revert the game state so we're not cloning objects
	# during loading. This will vary wildly depending on the needs of a
	# project, so take care with this step.
	# For our example, we will accomplish this by deleting saveable objects.
	var save_nodes = get_tree().get_nodes_in_group(_street_manager.STREET_GROUP)
	for i in save_nodes:
		remove_child(i)

	save_nodes = get_tree().get_nodes_in_group(_district_manager.DISTRICT_GROUP)
	for i in save_nodes:
		remove_child(i)

	save_nodes = get_tree().get_nodes_in_group(_intersection_manager.INTERSECTION_GROUP)
	for i in save_nodes:
		remove_child(i)
		
	# Load the file line by line and process that dictionary to restore
	# the object it represents.
	save_game.open("user://savegame.json", File.READ)
	while save_game.get_position() < save_game.get_len():
		# Get the saved dictionary from the next line in the save file
		var node_data = parse_json(save_game.get_line())
		
		# Preload all game objects 
		for i in node_data.keys():
			if i == "districts":
				for district in node_data[i]:
					_district_manager.load_district(district)
					
			if i == "intersections":
				for intersection in node_data[i]:
					_intersection_manager.load_intersection(intersection)
			
			if i == "streets":
				for street in node_data[i]:
					_street_manager.preload_street(street)
					
		# Update all references between game objects
		for i in node_data.keys():
			if i == "streets":
				for street in node_data[i]:
					_street_manager.load_street(street)

	save_game.close()

			
		
	
func _input(event):
	if event is InputEventKey:
		if event.pressed and event.scancode == KEY_F2:
			_save()
		if event.pressed and event.scancode == KEY_F3:
			_load()			
			
		if event.pressed and event.scancode == KEY_ESCAPE:
			get_tree().quit()		
						
				

func _on_build_street_toggled(button_pressed):
	if button_pressed:
		mode = BUILD_MODE.STREET
		$CanvasLayer/GUI/HBoxContainer/Btn_Marketplace.pressed = false

func _on_build_marketplace_toggled(button_pressed):
	if button_pressed:
		mode = BUILD_MODE.BUILDING
		$CanvasLayer/GUI/HBoxContainer/Btn_Street.pressed = false

