class_name EntityManager
extends Node

# ==============================================================================

signal created(entity)
signal deleted(entity)

# ==============================================================================

var entity_group : String = ""

var _mouse_world_position = Vector2(0, 0)

func preload_entity(data):
	pass

func load_entity(data):
	pass

func save():
	pass
	
func create(type = null):
	pass
	
func delete(entity, emit = true):
	#assert(get_all().has(entity))
	
	entity.remove_from_group(entity_group)	
	entity.queue_free()
	
	assert(not get_all().has(entity))
	
	if not emit:
		return 
		
	emit_signal("deleted", entity)

func get_all(filter: Array = []):
	assert(not entity_group.empty())
	
	#TODO check if this is really necessary
	var valid = []
	
	for entity in get_tree().get_nodes_in_group(entity_group):
		for filtered in filter:
			if entity == filtered:
				continue
				
		if is_instance_valid(entity):
			valid.push_back(entity)
			
	return valid
	#return get_tree().get_nodes_in_group(entity_group)
	
func get_by_id(id: int):
	assert(not entity_group.empty())
	assert(id >= 0)
	
	for node in get_all():
		if node.get_id() == id:
			return node
	
	return null

func _input(event):	
	if event is InputEventMouseMotion or event is InputEventMouseButton:
		_mouse_world_position = get_viewport().canvas_transform.affine_inverse().xform(event.position)
