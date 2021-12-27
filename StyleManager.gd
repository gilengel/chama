class_name StyleManager
extends Node

const FILE_NAME = "res://style.json"

enum Colors {
	Street,
	Outline,
	Background,
	Debug,
	District,
	Error
}

var _colors : Dictionary = {}

func _ready():
	var style = File.new()
	assert(style.file_exists(FILE_NAME))
	
	style.open(FILE_NAME, File.READ)
	
	while style.get_position() < style.get_len():
		var node_data = parse_json(style.get_line())
 
		for i in node_data.keys():	
			_colors[i] = Color(node_data[i])

func get_color(type: int) -> Color:
	return _colors[Colors.keys()[type].to_lower()]

