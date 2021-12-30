extends State

onready var _district_manager = get_node("../../DistrictManager")
onready var _building_manager = get_node("../../BuildingsManager")

var temp_building : Building = null
var temp_street : Street = null
var temp_district : District = null

#func _change_temp_building(building : Buildable):
#	destroy_enabled = false
#
#	if building:
#		enabled = true
#
#		remove_child(temp_building)
#		temp_building = building.duplicate()
#		temp_building.visible = false
#		add_child(temp_building)
#	else:
#		enabled = false
#
#		remove_child(temp_building)

func _get_influenced_districts(district, max_recursion = 1, _result = [], iteration = 0):
	if iteration == max_recursion:
		if not _result.has(district):
			_result.push_back(district)
		return _result
		
		
	for i in range(district.neighbours.size()):
		if not _result.has(district.neighbours[i]):
			_result.push_back(district.neighbours[i])
		
		
	for neighbour in district.neighbours:
		var influenced = _get_influenced_districts(neighbour, max_recursion, _result, iteration + 1)
		for i in range(influenced.size()):
			if not _result.has(influenced[i]):
				_result.push_back(influenced[i])
				
	return _result

# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseMotion:
		temp_district = null
		
		for district in _district_manager.get_all():
			district.set_hovered(false)
			
		for district in _district_manager.get_all():
			if district.is_point_in_district(_mouse_world_position):
				
				for influenced in _get_influenced_districts(district, temp_building.influence()):
					influenced.hover_color = Color.orange
					influenced.set_hovered(true)

				district.hover_color = Color.orangered
				district.set_hovered(true)
				temp_district = district
	
					
	if _event.is_action_pressed("place_object") and temp_building.is_constructable() and temp_district:	
		#temp_building.position = ExtendedGeometry.centroid_polygon_2d(temp_district.get_points())
		var new_building = temp_building.duplicate()
		new_building.district = temp_district
		new_building.visible = true
		
		add_child(new_building)
		
		
		


# Virtual function. Corresponds to the `_process()` callback.
func update(_delta: float) -> void:
	pass


# Virtual function. Called by the state machine upon changing the active state. The `msg` parameter
# is a dictionary with arbitrary data the state can use to initialize itself.
func enter(_msg := {}) -> void:
	assert(_msg.has("building"))
	
	temp_building  = _building_manager.create(_msg.building)



# Virtual function. Called by the state machine before changing the active state. Use this function
# to clean up the state.
func exit() -> void:
	_building_manager.delete(temp_building)
	temp_building = null



