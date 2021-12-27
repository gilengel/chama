extends Node2D

# ==============================================================================

export var PERSIST_GROUP = "Persist"

# ==============================================================================

onready var _street_manager = get_node("StreetManager")
onready var _district_manager = get_node("DistrictManager")
onready var _intersection_manager = get_node("IntersectionManager")
onready var _building_manager = get_node("BuildingsManager")
onready var _style_manager = get_node("StyleManager")

onready var Building = get_node("Building")

# ==============================================================================

func _ready():
	
	VisualServer.set_default_clear_color(_style_manager.get_color(StyleManager.Colors.Background))

	get_tree().connect("files_dropped", self, "_files_dropped")

func _files_dropped(files: PoolStringArray, screen: int):
	for file in files:
		var image = Image.new()
		var err = image.load(file)
		if err != OK:
			pass
			# Failed
		var texture = ImageTexture.new()
		texture.create_from_image(image, 0)
		
		var node = TextureRect.new()
		node.texture = texture
		

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

	update()

func _on_build_mode_change(mode, param):	
	var BUILDING_MODES = $CanvasLayer/GUI/HBoxContainer/MainPanel.BUILDING_MODES
	
	match mode:
		BUILDING_MODES.Building:
			$BuildingStateMachine.transition_to("CreateBuilding", {"building" : param })
		
		BUILDING_MODES.Street:
			$BuildingStateMachine.transition_to("StartCreateStreet")
			
		BUILDING_MODES.Destroy:
			$BuildingStateMachine.transition_to("Destroy")

#func _draw():
#	draw_polyline(_district_manager._outer_boundary, Color.orange, 40)
