extends Node2D

onready var _intersection_manager : IntersectionManager = get_node("../IntersectionManager")

func _ready():
	_intersection_manager.connect("intersection_created", self, "_foo")

func _foo(intersection):

	update()

func _draw():
	pass
		
