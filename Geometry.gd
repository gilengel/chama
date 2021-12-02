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
