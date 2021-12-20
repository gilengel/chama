extends State

onready var _intersection_manager = get_node("../../IntersectionManager")
onready var _street_manager = get_node("../../StreetManager")
onready var _district_manager = get_node("../DistrictManager")

var _starting_street
var _starting_intersection
var _valid_street

const SNAP_DISTANCE = 25

var start_position : Vector2 = Vector2(0, 0)

func _starts_on_street(point):
	for node in _street_manager.get_all():
		if Geometry.is_point_in_polygon(point, node.global_polygon()):
			return node
	
	return null
	
func _update_temp_street_start(position: Vector2):
	_starting_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
		
	if _starting_intersection:
		start_position = _starting_intersection.position
	else:
		_starting_street = _starts_on_street(position)
						
		if _starting_street:					
			var lambda = ((_starting_street.norm.x * (position.x - _starting_street.start.position.x)) + 
						  (_starting_street.norm.y * (position.y - _starting_street.start.position.y)))
						
			position = Vector2((_starting_street.norm.x * lambda) + _starting_street.start.position.x, 
							   (_starting_street.norm.y * lambda) + _starting_street.start.position.y)
							
		start_position = position			


	
# Virtual function. Receives events from the `_unhandled_input()` callback.
func handle_input(_event: InputEvent) -> void:
	.handle_input(_event)
	
	if _event is InputEventMouseButton:
		if _event.is_action_pressed("place_object"):		
			_update_temp_street_start(_mouse_world_position)
			var street = _street_manager.create()
			var start : Intersection = _intersection_manager.is_near_intersection(start_position, SNAP_DISTANCE)
			
			var split = _starting_street and not start
			
			if not start:
				start = _intersection_manager.create()
				start.global_position = start_position
			
			var split_street 
			if split:
				var _end = _starting_street.end
				_starting_street.end = start
				split_street = _street_manager.create()
				split_street.start = start
				split_street.end = _end

			var end = _intersection_manager.create()
			end.global_position = start_position
			
			street.start = start
			street.end = end
			
			
			state_machine.transition_to("EndCreateStreet", {
				street = street,
				start_position = start_position,
				start_splitted = [_starting_street, split_street] if split_street else []
			})
