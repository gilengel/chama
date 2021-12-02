extends MarginContainer

onready var statistics_label = $HBoxContainer/HBoxContainer/MarginContainer/RichTextLabel

var num_streets = 0
var num_intersections = 0
var num_houses = 0
var num_districts = 0
# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass


func _on_street_count_changed(count):
	num_streets = count
	
	statistics_label.text = "Streets: %s\nIntersections: %s\nHouses: %s\nDistricts: %s" % [
		num_streets,
		num_intersections,
		num_houses,
		num_districts
	]


func _on_DistrictManager_district_count_changed(count):
	num_districts = count
	
	statistics_label.text = "Streets: %s\nIntersections: %s\nHouses: %s\nDistricts: %s" % [
		num_streets,
		num_intersections,
		num_houses,
		num_districts
	]
