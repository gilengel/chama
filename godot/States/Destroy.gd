extends State

# ==============================================================================

onready var _street_manager = get_node("../../StreetManager")
onready var _building_manager = get_node("../../BuildingsManager")
onready var _intersection_manager = get_node("../../IntersectionManager")

# ==============================================================================

var temp_buildable : Buildable

# ==============================================================================

# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseMotion:	

		if temp_buildable:
			temp_buildable.set_hovered(false)
			temp_buildable.update()

		temp_buildable =  _street_manager.is_point_on_street(_mouse_world_position)
		
		if not temp_buildable:
			temp_buildable  = _building_manager.is_point_on_building(_mouse_world_position)
		
		if temp_buildable:
			temp_buildable.set_hovered(true)			
			temp_buildable.update()

	if _event.is_action_pressed("place_object") and temp_buildable:
		if temp_buildable is Street:
			var street = temp_buildable
			_street_manager.delete(street)
			
			if street.start._streets.size() == 2:
				var norm_1 = street.start._streets[0].street.norm
				var norm_2 = street.start._streets[1].street.norm
				
				var street_2 = street.start._streets[1].street
				
				var end = street_2.end
				street.start._streets[0].street.set_end(end)
				_intersection_manager.delete(street.start)
				_street_manager.delete(street_2)
				
				

		if temp_buildable is Building:
			_building_manager.delete(temp_buildable)
			
		temp_buildable = null
		
# Virtual function. Corresponds to the `_process()` callback.
func update(_delta: float) -> void:
	pass


# Virtual function. Called by the state machine upon changing the active state. The `msg` parameter
# is a dictionary with arbitrary data the state can use to initialize itself.
func enter(_msg := {}) -> void:
	pass


# Virtual function. Called by the state machine before changing the active state. Use this function
# to clean up the state.
func exit() -> void:
	pass
