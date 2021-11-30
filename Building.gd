class_name Building
extends Buildable

const MAX_FLOAT = 1.79769e308

func min_area():
	pass

func max_area():
	return MAX_FLOAT

func area():
	var sum = 0
	var size = polygon.size()
	for i in range(0, size):
		var p0 = polygon[i]
		var p1 = polygon[i+1] if i < size - 1 else polygon[0]

		sum += p0.x * p1.y - p1.x * p0.y

	return 0.5 * abs(sum)

