extends Node2D

# ==============================================================================

export var PERSIST_GROUP = "Persist"

# ==============================================================================

onready var _street_manager = get_node("StreetManager")
onready var _district_manager = get_node("DistrictManager")
onready var _intersection_manager = get_node("IntersectionManager")
onready var _building_manager = get_node("BuildingsManager")

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

	var p = [Vector2(1172, 729), Vector2(907, 608), Vector2(834, 729), Vector2(932, 795), Vector2(843, 909), Vector2(775, 726), Vector2(868, 586), Vector2(707, 609), Vector2(697, 911), Vector2(589, 830), Vector2(654, 627), Vector2(378, 736), Vector2(336, 575), Vector2(515, 607), Vector2(586, 474), Vector2(398, 471), Vector2(512, 309), Vector2(682, 406), Vector2(697, 335), Vector2(780, 337), Vector2(679, 104), Vector2(853.458313, 91.054321), Vector2(937.404724, 190.474731), Vector2(907, 188), Vector2(899, 328), Vector2(1015.37146, 282.813202), Vector2(1098.253906, 380.973572), Vector2(925, 499), Vector2(925, 499), Vector2(1149, 604), Vector2(1200, 515), Vector2(1196.455322, 497.276611), Vector2(1300.5, 620.5), Vector2(1172, 729)]
	p = [Vector2(1172, 197), Vector2(1173, 374), Vector2(547, 444), Vector2(566, 617), Vector2(1192, 626), Vector2(1206, 847), Vector2(835, 837), Vector2(840.823425, 620.951172), Vector2(1192, 626), Vector2(1206, 847), Vector2(464, 827), Vector2(299, 484), Vector2(501, 209)]
	#p = [Vector2(835, 837), Vector2(464, 827), Vector2(299, 484), Vector2(501, 209), Vector2(1172, 197), Vector2(1173, 374), Vector2(547, 444), Vector2(566, 617), Vector2(840.823425, 620.951172), Vector2(835, 837)]
	p.invert()
	#for i in range(p.size()-1):
	#	_street_manager.create_street(p[i], p[i+1])
	
	
	var size = 300
#	_street_manager.create_street(Vector2(mx-size, my-size), Vector2(mx-size, my+size))
#	_street_manager.create_street(Vector2(mx-size, my+size), Vector2(mx+size, my+size))
#	_street_manager.create_street(Vector2(mx+size, my+size), Vector2(mx+size, my))
#	_street_manager.create_street(Vector2(mx+size, my), Vector2(mx - size / 2, my + size / 2))
#	_street_manager.create_street(Vector2(mx - size / 2, my + size / 2), Vector2(mx, my - size))
#	_street_manager.create_street(Vector2(mx, my - size), Vector2(mx-size, my-size))
	#_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx+150, my-150))
	

	
#	_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx+150, my-150))
#	_street_manager.create_street(Vector2(mx+150, my-150), Vector2(mx+150, my+150))
#	_street_manager.create_street(Vector2(mx+150, my+150), Vector2(mx-150, my+150))
#	_street_manager.create_street(Vector2(mx-150, my+150), Vector2(mx-150, my-150))

	# default street
#	_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx+150, my-150))
#	#_street_manager.create_street(Vector2(mx+150, my-150), Vector2(mx+150, my+150))
#	_street_manager.create_street(Vector2(mx+150, my+150), Vector2(mx-150, my+150))
#	_street_manager.create_street(Vector2(mx-150, my+150), Vector2(mx-150, my-150))
#
#	_street_manager.create_street(Vector2(mx+150, my-150), Vector2(mx+300, my-150))
#	_street_manager.create_street(Vector2(mx+300, my+150), Vector2(mx+300, my-150))
#
#
#	_street_manager.create_street(Vector2(mx+150, my+150), Vector2(mx+300, my+150))
#	_street_manager.create_street(Vector2(mx+300, my+150), Vector2(mx+300, my+450))
#	_street_manager.create_street(Vector2(mx+150, my+450), Vector2(mx+300, my+450))
#	_street_manager.create_street(Vector2(mx+150, my+450), Vector2(mx+150, my+150))
#
#	_street_manager.create_street(Vector2(mx+300, my-150), Vector2(mx+500, my-150))
#	_street_manager.create_street(Vector2(mx+300, my+150), Vector2(mx+500, my+150))
#
#	_street_manager.create_street(Vector2(mx-150, my-150), Vector2(mx-300, my-150))
#	_street_manager.create_street(Vector2(mx-150, my+150), Vector2(mx-300, my+150))
#	_street_manager.create_street(Vector2(mx-300, my-150), Vector2(mx-300, my+150))
#
#	_street_manager.create_street(Vector2(mx-300, my-150), Vector2(mx-300, my-450))
#	_street_manager.create_street(Vector2(mx-300, my-450), Vector2(mx-150, my-450))
#	_street_manager.create_street(Vector2(mx-150, my-450), Vector2(mx-150, my-150))
func _save():
	var save_game = File.new()
	save_game.open("user://savegame.json", File.WRITE)

	var streets = []
	for street in _street_manager.get_all():
		streets.append(street.call("save"))
	
	var districts = []
	for district in _district_manager.get_all():
		districts.append(district.call("save"))

	var intersections = []
	for intersection in _intersection_manager.get_all():
		intersections.append(intersection.call("save"))
		
	var buildings = []
	for building in _building_manager.get_all():
		buildings.append(building.call("save"))
	
	var save_dict = {
		"streets": streets,
		"districts": districts,
		"intersections": intersections,
		"buildings": buildings		
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
	var save_nodes = _street_manager.get_all()
	for i in save_nodes:
		remove_child(i)

	save_nodes = _district_manager.get_all()
	for i in save_nodes:
		remove_child(i)

	save_nodes = _intersection_manager.get_all()
	for i in save_nodes:
		remove_child(i)
		
	save_nodes = _building_manager.get_all()
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
					_district_manager.preload_entity(district)
					
			if i == "intersections":
				for intersection in node_data[i]:
					_intersection_manager.load_entity(intersection)
			
			if i == "streets":
				for street in node_data[i]:
					_street_manager.preload_entity(street)
					
			if i == "buildings":
				for building in node_data[i]:
					_building_manager.preload_entity(building)
					
		# Update all references between game objects
		for i in node_data.keys():
			if i == "streets":
				for street in node_data[i]:
					_street_manager.load_entity(street)
					
			if i == "districts":
				for district in node_data[i]:
					_district_manager.load_entity(district)
					
			if i == "buildings":
				for building in node_data[i]:
					_building_manager.load_entity(building)

	save_game.close()

			
		
	
func _input(event):
	if event is InputEventKey:
		if event.pressed and event.scancode == KEY_F2:
			_save()
		if event.pressed and event.scancode == KEY_F3:
			_load()	
			
		if event.pressed and event.scancode == KEY_M:
			var districts = _district_manager.get_all()
			for i in range(districts.size()):
				districts[i].splits += 1
				districts[i].update()

		if event.pressed and event.scancode == KEY_N:
			var districts = _district_manager.get_all()
			for i in range(districts.size()):
				if districts[i].splits > 1:
					districts[i].splits -= 1
					districts[i].update()
			
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
