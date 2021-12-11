class_name House
extends Building

var rng = RandomNumberGenerator.new()

var houses = []

func get_ui_name():
	return "House"
	
func min_area():
	return pow(200, 2)
	
func influence():
	return 0	
	
func set_district(new_district):
	.set_district(new_district)
	
	houses = generate_houses(new_district.polygon)
	update()
	
func _ready():
	._ready()
	rng.randomize()	
	
func is_constructable():
	return true
	
	

	
func _line_segment(v1: Vector2, v2: Vector2, cw) -> Dictionary:
		var vec = (v2 - v1)
		var norm = vec.normalized()
		var perp = Vector2(-norm.y, norm.x)	* (1 if cw else -1)
			
		return { "s": v1, "e": v2, "n": norm, "p": perp, "l": vec.length()}

		
func generate_houses(polygon: Array):
	var polygons = []
	
	
	#if _splits == splits:
	if ExtendedGeometry.area_polygon_2d(polygon) < min_area():
		polygons.push_back(polygon)
#		polygons.append({ 
#			"p": polygon, "c": Color(rng.randf(), rng.randf(), rng.randf(), 0.7),
#			"pp": parent, "pc": Color(rng.randf(), rng.randf(), rng.randf(), 1),
#			"i": ints
#		})
		
		return polygons
		
	var polygon_size = polygon.size()	
	var values = []
	
	var cw = Geometry.is_polygon_clockwise(polygon)
	for i in range(polygon_size):
		values.append(_line_segment(polygon[i], polygon[(i+1) % polygon_size], cw))	
	
	var longest_street = _get_longest_side(values, [])
	var midpoint = _get_split_point(values[longest_street])
	var intersections = _calculate_intersections(longest_street, midpoint, values, cw)
	
	for sub_poly in _calculate_split_polygons(polygon, intersections):
		polygons.append_array(generate_houses(sub_poly))
		
	return polygons
	
	
	
func _get_split_point(segment: Dictionary) -> Vector2:
	return segment.s + segment.n * segment.l * rng.randf_range(0.3, 0.7)
	
func _get_longest_side(values: Array, is_street: Array) -> int:
	# first filter side that are connected to a street because they
	# will be prioritized
	var values_with_streets = []
		
	var _values = values if values_with_streets.empty() else values_with_streets
	
	var longest_side = _values[0].l
	var longest_index = 0	
	for i in range(1, _values.size()):
		if _values[i].l > longest_side:
			longest_side = _values[i].l
			longest_index = i
	
	return longest_index		

static func _sort_intersections(a, b) -> bool: 
	return a.point.length() < b.point.length()	
	
func _calculate_intersections(longest_index: int, midpoint: Vector2, values: Array, cw: bool) -> Array:
	var endpoint = midpoint - values[longest_index].p * 60000

	var intersections = []
	
	intersections.push_back({ "index": longest_index, "point": Vector2(0, 0) })	
	for i in range(values.size()):
		if i == longest_index:
			continue
		
		var intersection = Geometry.segment_intersects_segment_2d(midpoint, endpoint, values[i].s, values[i].e)
		
		if not intersection:
			continue
			
		var ignore = false
		for j in range(values.size()):
			if intersection == values[j].s:
				ignore = true
				break
				
				
		if not ignore:			
			intersections.push_back({ "index": i, "point": intersection - midpoint })
		
	for i in range(intersections.size()):
		intersections[i].point += midpoint
	
	
	intersections.sort_custom(self, "_sort_intersections")
				
	return intersections
		
	
func _calculate_split_polygons(polygon: Array, intersections: Array) -> Array:
	for i in range(polygon.size() - 1):
		assert(polygon[i] != polygon[i+1])

	var result = []
	var result_indices = []

	var intersection_pairs = []
	for i in range(0, intersections.size()-1, 2):
		intersection_pairs.push_back({ "entry": intersections[i], "exit": intersections[i+1]})

	var current_index = 0
	result.push_back({ "points": [], "crossback": null })
	result_indices.push_back([])
	for i in range(polygon.size()):
		result[current_index].points.push_back(polygon[i])
		result_indices[current_index].push_back(i)

		for j in range(intersections.size()):
			if i == intersections[j].index:

				var other_point = null
				for k in range(intersection_pairs.size()):
					if intersections[j].index == intersection_pairs[k].entry.index:
						other_point = intersection_pairs[k].exit.index
					if intersections[j].index == intersection_pairs[k].exit.index:
						other_point = intersection_pairs[k].entry.index

				result[current_index].points.push_back(intersections[j].point)
				result_indices[current_index].push_back("s%s" % j)

				result[current_index].crossback = other_point

				var existed = false
				for k in range(result.size()):
					if k == current_index:
						continue

					if result[k].crossback == intersections[j].index:					
						current_index = k
						existed = true

				if not existed:
					result.push_back({ "points": [], "crossback": j })
					result_indices.push_back([])
					current_index = result.size() - 1

				result[current_index].points.push_back(intersections[j].point)
				result_indices[current_index].push_back("s%s" % j)
	
	# remove the crossback information
	var _result = []
	for poly in result:
		_result.push_back(poly.points)

	return _result	
	
func _draw():
	for house in houses:
		
		draw_polyline(house, Color.black, 16)

