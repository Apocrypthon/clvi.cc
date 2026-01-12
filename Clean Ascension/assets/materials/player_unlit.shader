shader_type spatial;
render_mode unshaded, cull_back;

uniform vec4 base_color : hint_color = vec4(1.0, 1.0, 1.0, 1.0);
uniform sampler2D base_albedo : hint_albedo;
uniform sampler2D mask_texture : hint_albedo;
uniform vec4 mask_color : hint_color = vec4(1.0, 1.0, 1.0, 1.0);

void fragment() {
	vec4 albedo = texture(base_albedo, UV);
	vec4 mask = texture(mask_texture, UV);
	vec3 mixed_color = mix(albedo.rgb * base_color.rgb, mask_color.rgb, mask.r);
	ALBEDO = mixed_color;
	ALPHA = albedo.a * base_color.a;
}
