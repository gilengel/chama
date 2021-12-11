class_name MainPanel
extends Control

# ==============================================================================

onready var _buildings_manager = get_node("/root/City/BuildingsManager")
onready var buildings = _buildings_manager.get_node("Buildings")
onready var streets = _buildings_manager.get_node("Streets")

# ==============================================================================

signal building_changed(building)
signal street_changed(street)
signal destroy_mode_changed(enabled)

# ==============================================================================

onready var buildings_tab = $HBoxContainer/Tab/Buildings
onready var streets_tab = $HBoxContainer/Tab/Streets

# ==============================================================================

var _btn_group : ButtonGroup = ButtonGroup.new()

# ==============================================================================

func _add_building_toggle_button(node: Buildable, tab: Control):
	var btn = Button.new()
	btn.toggle_mode = true
	btn.text = node.get_ui_name()
	btn.group = _btn_group
	btn.set_meta("building", node)
	
	btn.connect("toggled", self, "_toggle_building")
	
	tab.add_child(btn)
	
func _add_street_toggle_button(node: Buildable, tab: Control):
	var btn = Button.new()
	btn.toggle_mode = true
	btn.text = node.get_ui_name()
	btn.group = _btn_group
	btn.set_meta("street", node)
	
	btn.connect("toggled", self, "_toggle_street")
	
	tab.add_child(btn)

func _toggle_building(toggled: bool):
	if toggled:
		var building = _btn_group.get_pressed_button().get_meta("building")
		
		emit_signal("destroy_mode_changed", false)
		emit_signal("street_changed", null)
		emit_signal("building_changed", building)
		
func _toggle_street(toggled: bool):
	if toggled:
		var street = _btn_group.get_pressed_button().get_meta("street")
		
		emit_signal("destroy_mode_changed", false)
		emit_signal("building_changed", null)
		emit_signal("street_changed", street)

func _add_building_buttons():
	for building in buildings.get_children():
		_add_building_toggle_button(building, buildings_tab)
		
func _add_street_buttons():
	for street in streets.get_children():
		_add_street_toggle_button(street, streets_tab)

func _ready():
	_add_building_buttons()
	_add_street_buttons()
	
	$HBoxContainer/Btn_Delete.group = _btn_group


func _on_Btn_Delete_toggled(button_pressed):
	if button_pressed:
		emit_signal("destroy_mode_changed", true)
