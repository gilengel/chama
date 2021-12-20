class_name IntersectionManager
extends EntityManager

onready var _street_manager = get_node("../StreetManager")

const INTERSECTION_GROUP = "Intersections"

signal intersection_count_changed(count)

func _ready():
	entity_group = INTERSECTION_GROUP
	
	_street_manager.connect("deleted", self, "_street_deleted")

func _street_deleted(street: Street):
	if street.start._streets.empty():
		delete(street.start)
		
	if street.end._streets.empty():
		delete(street.end)

func delete(entity, emit = true):
	.delete(entity)
	
	if not emit:
		return
		
	emit_signal("intersection_count_changed", get_all().size())
	
func load_entity(data):
	var intersection = Intersection.new()
	
	intersection.position = Vector2(data.pos_x, data.pos_y)
	intersection.add_to_group(INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)
	
	intersection.set_id(data.id)

func create_intersection(position):
	var intersection = Intersection.new()
	intersection.position = position
	intersection.add_to_group(INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)
	
	emit_signal("intersection_count_changed", get_all().size())
	
	return intersection
	
func create(type = null):
	var intersection = Intersection.new()
	intersection.add_to_group(INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)
	
	emit_signal("intersection_count_changed", get_all().size())
	
	return intersection	

func is_near_intersection(point, allowed_distance, ignored_intersections = []) -> Intersection:
	
	for node in get_tree().get_nodes_in_group(INTERSECTION_GROUP):
		if ignored_intersections.has(node): 
			continue;

		if node.position.distance_to(point) < allowed_distance:
			return node
	
	return null	
