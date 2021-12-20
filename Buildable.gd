class_name Buildable
extends Polygon2D

var hovered = false setget set_hovered
var hover_color : Color = Color.olivedrab
var normal_color : Color = Color.black setget set_normal_color
var error_color : Color = Color.orangered


onready var _id = get_index() setget set_id, get_id  

func shape():
	pass

func _ready():
	color = normal_color

func set_hovered(new_value : bool):
	if new_value:
		color = hover_color
	else:
		color = normal_color
		
func set_normal_color(new_normal_color : Color):
	pass
	
func get_ui_name():
	pass

func is_constructable():
	pass

func set_id(id):
	_id = id
	
func get_id():
	return _id
