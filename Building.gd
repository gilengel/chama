class_name Building
extends Buildable

const MAX_FLOAT = 9999999999999

func min_area():
	pass

func max_area():
	return MAX_FLOAT

# formula used from https://en.wikipedia.org/wiki/Centroid#Of_a_polygon
func centroid():
	var sum_x : float = 0.0
	var sum_y : float = 0.0
	var area = area()
	
	
	
	if area == 0:
		return Vector2(0, 0)
	
	var size = polygon.size()
	for i in range(0, size):
		var p0 = polygon[i]
		var p1 = polygon[i+1] if i < size - 1 else polygon[0]
		
		var term = (p0.x * p1.y - p1.x * p0.y)
		sum_x += (p0.x + p1.x) * term
		sum_y += (p0.y + p1.y) * term

	
	var x = 1.0 / (6.0 * area)  * sum_x
	var y = 1.0 / (6.0 * area)  * sum_y
	
	return Vector2(x, y)
	
func area():
	var sum = 0
	var size = polygon.size()
	for i in range(0, size):
		var p0 = polygon[i]
		var p1 = polygon[i+1] if i < size - 1 else polygon[0]

		sum += p0.x * p1.y - p1.x * p0.y

	return 0.5 * abs(sum)

func is_constructable():
	var area = area()
	
	return area >= min_area() and area <= max_area()
