use super::Transform2D;

pub fn to_matrix_sfml(transform: &Transform2D) -> sfml::graphics::Transform {
    let angle = transform.rotation.as_rad();
    let angle = -angle;
    let (sine, cosine) = angle.sin_cos();
    let sxc = transform.scale.x * cosine;
    let syc = transform.scale.y * cosine;
    let sxs = transform.scale.x * sine;
    let sys = transform.scale.y * sine;
    let tx = transform.position.x;
    let ty = transform.position.y;

    // R | 0
    // T | 1
    sfml::graphics::Transform::new(sxc, sys, tx, -sxs, syc, ty, 0.0, 0.0, 1.0)
}
