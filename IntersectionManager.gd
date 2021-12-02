class_name IntersectionManager
extends Node

const INTERSECTION_GROUP = "Intersections"

signal intersection_created(intersection)

func load_intersection(data):
	var intersection = Intersection.new()
	
	intersection.position = Vector2(data.pos_x, data.pos_y)
	intersection.add_to_group(INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)
	
	intersection.set_id(data.id)

func get_intersections():
	return get_tree().get_nodes_in_group(INTERSECTION_GROUP)

func get_intersection_by_id(id):
	for node in get_intersections():
		if node.get_id() == id:
			return node
	
	return null

func create_intersection(position):
	var intersection = Intersection.new()
	intersection.position = position
	intersection.add_to_group(INTERSECTION_GROUP)
	intersection.add_to_group($"../".PERSIST_GROUP)
	add_child(intersection)
	
	emit_signal("intersection_created", intersection)
	
	return intersection

func is_near_intersection(point, allowed_distance, ignored_intersections = []) -> Intersection:
	
	for node in get_tree().get_nodes_in_group(INTERSECTION_GROUP):
		if ignored_intersections.has(node): 
			continue;

		if node.position.distance_to(point) < allowed_distance:
			return node
	
	return null	
