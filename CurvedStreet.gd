class_name CurvedStreet
extends Street

func get_class() -> String:
	return "CurvedStreet"
	
func get_ui_name():
	return "Curved Street"
	
func _ready():
	update()
	
	._ready()
