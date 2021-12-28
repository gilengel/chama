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
		
func delete(street, emit = true):
	if emit:
		emit_signal("street_deleted", street)
		
	if street.start:
		street.start.remove_street(street)
	
	if street.end:
		street.end.remove_street(street)
	
	.delete(street)
	
	if emit:
		emit_signal("street_count_changed", get_all().size())
	
# Called when the node enters the scene tree for the first time.
func _ready():	
	entity_group = STREET_GROUP
	_gui_main_panel.connect("street_changed", self, "_change_temp_street")
	_gui_main_panel.connect("destroy_mode_changed", self, "_enable_destroy")
	
func _enable_destroy(value):
	if value:
		enabled = false
	
func is_point_on_street(pt : Vector2, ignored : Array = []) -> Street :
	for s in get_tree().get_nodes_in_group(STREET_GROUP):
		if ignored.find(s) != -1:
			continue
			
		if Geometry.is_point_in_polygon(pt, s.global_polygon()):
			return s
			
	return null
				
func count_intersections_with_line(start: Vector2, dir: Vector2) -> Array:
	var end = start + dir * 99999
	
	var count = 0
	var result = []
	for street in get_all():
		if Geometry.segment_intersects_segment_2d(street.start.global_position, street.end.global_position, start, end):
			count += 1
			result.push_back(street.get_id())
			
	#return count
	return result

func get_new_id():
	var highest_id = 0
	for s in get_all():
		if s.get_id() > highest_id:
			highest_id = s.get_id()	
			
	return highest_id + 1	

func create(type = null):
	var street = Street.new()
	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)
	

	var id = get_new_id()
	add_child(street)
	street._id = id
	
	emit_signal("street_count_changed", get_all().size())
	
	return street

func create_curved():
	var street = CurvedStreet.new()
	street.add_to_group(STREET_GROUP)
	street.add_to_group($"../".PERSIST_GROUP)	

	var id = get_new_id()
	add_child(street)
	street._id = id
	
	emit_signal("street_count_changed", get_all().size())
	
	return street	
