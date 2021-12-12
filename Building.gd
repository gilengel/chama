class_name Building
extends Buildable

const MAX_FLOAT = 9999999999999

var district : District = null setget set_district

# We need to overwrite it so that we can instance new objects later one
# only based on the class name
func get_class(): return get_ui_name()

func influence():
	return 1

func min_area():
	pass

func max_area():
	return MAX_FLOAT
	
func set_district(new_district : District):
	district = new_district

# formula used from https://en.wikipedia.org/wiki/Centroid#Of_a_polygon
func centroid():
	return ExtendedGeometry.centroid_polygon_2d(polygon)

func is_constructable():
	var area = ExtendedGeometry.area_polygon_2d(polygon)
	
	return area >= min_area() and area <= max_area()

func save():
	var save_dict = {
		"id": get_id(),
		"type": get_ui_name(),
		"district": district.get_id()
	}
	
	return save_dict
	
