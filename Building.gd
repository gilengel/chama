class_name Building
extends Buildable

const MAX_FLOAT = 9999999999999

var district : District = null

func min_area():
	pass

func max_area():
	return MAX_FLOAT

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
	
