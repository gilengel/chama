class_name StreetManager
extends EntityManager

# ==============================================================================





const STREET_GROUP = "Streets"

# ==============================================================================

var Street = preload("res://Street.gd")
var Intersection = preload("res://Intersection.gd")
var District = preload("res://District.gd")

# ==============================================================================

signal street_count_changed(count)



signal street_deleted(street)

# ==============================================================================

onready var _game_state_manager = get_node("../../City")
onready var _district_manager = get_node("../DistrictManager")
onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")
onready var _camera = get_node("../Camera2D")

onready var _gui_main_panel : MainPanel = get_node("/root/City/CanvasLayer/GUI/HBoxContainer/MainPanel")

# ==============================================================================

enum State {NOTHING, START_STREET, END_STREET}
var state = State.NOTHING


var _starting_intersection
var _valid_street = false

var enabled = false




# ==============================================================================


func preload_entity(data):
	var street = Street.new()

	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	street.update()
	
	add_child(street)
	
	# must be after add_child in order to overwrite id
	street.set_id(data.id)

func load_entity(data):
	var street = get_by_id(data.id)
		
	street.set_start(_intersection_manager.get_by_id(data.start))
	street.set_end(_intersection_manager.get_by_id(data.end))	
	
	if data.d_l as float >= 0:
		street.left_district =  _district_manager.get_by_id(data.d_l as float)
	if data.d_r as float  >= 0:
		street.right_district =  _district_manager.get_by_id(data.d_r as float)
		
func delete(street):
	street.start.remove_street(street)
	street.end.remove_street(street)
	
	emit_signal("street_deleted", street)
	
	.delete(street)
	
	emit_signal("street_count_changed", get_all().size())
	
# Called when the node enters the scene tree for the first time.
func _ready():	
	entity_group = STREET_GROUP
	_gui_main_panel.connect("street_changed", self, "_change_temp_street")
	_gui_main_panel.connect("destroy_mode_changed", self, "_enable_destroy")
	
func _enable_destroy(value):
	if value:
		enabled = false
	
func _change_temp_street(street : Buildable):
	if street:
		enabled = true
	else:
		enabled = false	
			
func is_point_on_street(pt : Vector2) -> Street :
	for s in get_tree().get_nodes_in_group(STREET_GROUP):
		if Geometry.is_point_in_polygon(pt, s.global_polygon()):
			return s
			
	return null
				



func create(type = null):
	var street = Street.new()
	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	add_child(street)
	
	return street

#func _intersect_with_street(position):
#	var near_intersection = _intersection_manager.is_near_intersection(position, SNAP_DISTANCE)
#	if near_intersection:
#		temp["end"] = near_intersection.position
#
#	var a = []
#	for s in get_tree().get_nodes_in_group(STREET_GROUP):
#		if _starting_street and s.get_index() == _starting_street.get_index():
#			continue;
#
#		var r = Geometry.segment_intersects_segment_2d(s.position, s.end.position, temp["start"], temp["end"])
#
#		if r:
#			a.append(r)
#
#
#	var shortest = Vector2(50000, 50000)
#	var shortest_distance = 80000000000
#
#	if a:				
#		for intersection in a:				
#			var distance = intersection.distance_squared_to(temp["start"]) 
#			if distance < shortest_distance:
#				shortest_distance = distance
#				shortest = intersection
#
#		temp["end"] = shortest
