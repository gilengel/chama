class_name Buildable
extends Polygon2D

var hovered = false setget set_hovered
var hover_color : Color = Color.olivedrab
var normal_color : Color = Color.black

func _ready():
	color = normal_color

func set_hovered(new_value : bool):
	if new_value:
		color = hover_color
	else:
		color = normal_color
		
func get_ui_name():
	pass

func is_constructable():
	pass
