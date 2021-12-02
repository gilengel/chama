class_name ExtendedGeometry
extends Node


static func deflate_polygon_2d(polygon: PoolVector2Array, offset: float):
	var new_poly = []
	var last : Vector2 = Vector2(0, 0)
	for i in range(0, polygon.size() - 1):
		var norm = (polygon[i+1] - polygon[i]).normalized()
		var perp_vec = Vector2(-norm.y, norm.x)
		
		new_poly.append(polygon[i] + perp_vec * -offset)
		new_poly.append(polygon[i+1] + perp_vec * -offset)
		

		
		last = polygon[i+1] + perp_vec * -offset

	return new_poly

static func area_polygon_2d(polygon: PoolVector2Array) -> float:
	var sum = 0
	var size = polygon.size()
	
	print(size)
	for i in range(0, size):
		var p0 = polygon[i]
		var p1 = polygon[i+1] if i < size - 1 else polygon[0]

		sum += p0.x * p1.y - p1.x * p0.y

	return 0.5 * abs(sum)
	
static func centroid_polygon_2d(polygon: PoolVector2Array) -> Vector2:
	var sum_x : float = 0.0
	var sum_y : float = 0.0
	var area = area_polygon_2d(polygon)
		
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
