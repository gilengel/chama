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
	
static func average_centroid_polygon_2d(polygon: PoolVector2Array) -> Vector2:
	var result = Vector2(0, 0)
	
	var size = polygon.size()
	for i in range(size):
		result += polygon[i]
		
	result /= size
	
	return result
		
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
	


static func _sort_by_x(a: Vector2, b: Vector2) -> bool:
	return a.x < b.x

static func order_polygon_2d_clockwise(polygon: PoolVector2Array) -> PoolVector2Array:
	var min_x = 8000000000
	var min_x_index = -1
	
	var max_x = -min_x
	var max_x_index = -1
	for i in range(polygon.size()):
		if polygon[i].x < min_x:
			min_x_index = i
			min_x = polygon[i].x
			
		if polygon[i].x > max_x:
			max_x_index = i
			max_x = polygon[i].x
			
	return Geometry.convex_hull_2d(polygon)

static func _decision_distance(p: Vector2, q: Array):
	var min_distance = 99999999
	
	var min_index = 1
	for i in range(q.size()):
		var distance = q[i].distance_to(p)
		
		if distance > 0 and distance < min_distance:
			min_distance = distance
			min_index = i
			
	return min_distance
	
static func _distance_to_edge(p: Vector2, edge: Array):
	return min(p.distance_to(edge[0]), p.distance_to(edge[1]))
	
static func _distance_to_triangle(p: Vector2, triangle: Array):
	return min(min(p.distance_to(triangle[0]), p.distance_to(triangle[1])), min(p.distance_to(triangle[1]), p.distance_to(triangle[2])))
	
static func _nearest_point_to_edge(polygon: PoolVector2Array, edge: Array):
	var distance = 99999999
	var point = polygon[0]
	for p in polygon:
		var p_distance = _distance_to_edge(p, edge)
		
		if p_distance > 0 and p_distance < distance:
			distance = p_distance
			point = p
	
	return point

static func concave_hull_2d(polygon: PoolVector2Array):
	var convex_hull = Geometry.convex_hull_2d(polygon)
		
	var convex_pairs = []
	
	for i in range(convex_hull.size() - 1):
		convex_pairs.push_back([convex_hull[i], convex_hull[i + 1]])
	
	var N = 1
	
	var concave_hull : Array = convex_pairs
	
	for i in range(1, concave_hull.size()-1):
		var edge = concave_hull[i]
		
		var p = _nearest_point_to_edge(polygon, edge)
		var eh = edge[0].distance_to(edge[1])
		var dd = _decision_distance(p, edge)
		
		if float(eh / dd) > N:
			concave_hull.erase(concave_hull[i])
			concave_hull.push_back([edge[0], p])	
			concave_hull.push_back([edge[1], p])
	
	var result = []
	for i in range(concave_hull.size()):
		result.push_back(concave_hull[i][0])
		result.push_back(concave_hull[i][1])
		
	return result
