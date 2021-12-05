class_name ExtendedGeometry
extends Node

class ClockwiseSorter:
	static func _angle(pt: Vector2) -> float:
		var angle = pt.angle()
		
		if angle <= 0:
			angle = 2 * PI + angle
		
		return angle
		
	static func _sort(a: Vector2, b: Vector2) -> bool:
		var a_angle = _angle(a)
		var b_angle = _angle(b)
				
		if a_angle < b_angle:
			return true
		
		var a_d = a.length()
		var b_d = b.length()
		
		if (a_angle == b_angle) and a_d < b_d:
			return true
			
		return false

static func area_polygon_2d(polygon: PoolVector2Array) -> float:
	var sum = 0
	var size = polygon.size()
	
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
	

	
	
static func order_polygon_2d_clockwise(polygon: PoolVector2Array) -> PoolVector2Array:
	var center : Vector2 = Vector2(0, 0)
	for pt in polygon:
		center += pt
	
	center /= polygon.size()
	
	var p2 = []
	for i in range(polygon.size()):
		p2.push_back(polygon[i] - center)
		
	p2.sort_custom(ClockwiseSorter, "_sort")
	
	for i in range(p2.size()):
		p2[i] += center
	
	return p2
