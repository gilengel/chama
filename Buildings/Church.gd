class_name Church
extends Building

func get_ui_name():
	return "Church"

func min_area():
	return 0

func _draw():
	var centroid = centroid()
	
	var width := 80
	var length := 200
	
	var hw = width / 2.0
	var hl = length / 2.0
	
	var small_w = width * 0.4
	var small_l = length * 0.25
	
	centroid.y += length * 0.1
	
	draw_line(centroid - Vector2(-hw, -hl), centroid - Vector2(hw, -hl), Color.black, 4)
	draw_rect(Rect2(centroid - Vector2(-hw, -hl + small_w), Vector2(small_w, small_w)), Color.black, false, 4)
	draw_rect(Rect2(centroid - Vector2(-hw, hl - small_l / 2.0), Vector2(small_w, length - small_w - small_l / 2.0)), Color.black, false, 4)

	draw_rect(Rect2(centroid - Vector2(hw + small_w, -hl + small_w), Vector2(small_w, small_w)), Color.black, false, 4)
	draw_rect(Rect2(centroid - Vector2(hw + small_w, hl - small_l / 2.0), Vector2(small_w, length - small_w - small_l / 2.0)), Color.black, false, 4)

	var long_w = width * 0.75
	var p = centroid - Vector2(hw + long_w, hl + small_l / 2.0)
	draw_line(p, p + Vector2(long_w, 0), Color.black, 4)
	draw_line(p + Vector2(0, small_l), p + Vector2(long_w, small_l), Color.black, 4)
	draw_line(p - Vector2(0, 2), p + Vector2(0, small_l + 2), Color.black, 4)
	
	
	p = centroid + Vector2(hw, -hl - small_l / 2.0)
	draw_line(p, p + Vector2(long_w, 0), Color.black, 4)
	draw_line(p + Vector2(0, small_l), p + Vector2(long_w, small_l), Color.black, 4)
	draw_line(p + Vector2(long_w, -2), p + Vector2(long_w, small_l + 2), Color.black, 4)
	
	#draw_rect(Rect2(centroid() - Vector2(hw + 80, hl + small_l / 2.0), Vector2(80, small_l)), Color.black, false, 4)
	#draw_rect(Rect2(centroid() - Vector2(-hw, hl + small_l / 2.0), Vector2(80, small_l)), Color.black, false, 4)
	
	draw_arc(centroid - Vector2(0, hl), hw * 1.1, 0, 2 * PI, 180, Color.black, 4)
