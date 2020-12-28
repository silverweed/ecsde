use crate::render::Vertex;
use inle_math::vector::Vec2f;

#[inline]
fn compute_normal(p1: Vec2f, p2: Vec2f) -> Vec2f {
    v2!(p1.y - p2.y, p2.x - p1.x).normalized_or_zero()
}

// Algorithm taken from sf::Shape::updateOutline()
pub fn create_outline(vertices: &[Vertex], outline_vertices: &mut Vec<Vertex>, thickness: f32) {
    let count = vertices.len() - 2;
    outline_vertices.resize((count + 1) * 2, Vertex::default());

    for i in 0..count {
        let index = i + 1;

        // Get the two segments shared by the current point
        let p0 = if i == 0 {
            vertices[count].position
        } else {
            vertices[index - 1].position
        };
        let p1 = vertices[index].position;
        let p2 = vertices[index + 1].position;

        // Compute their normal
        let mut n1 = compute_normal(p0, p1);
        let mut n2 = compute_normal(p1, p2);

        // Make sure that the normals point towards the outside of the shape
        // (this depends on the order in which the points were defined)
        if n1.dot(vertices[0].position - p1) > 0. {
            n1 = -n1;
        }

        if n2.dot(vertices[0].position - p1) > 0. {
            n2 = -n2;
        }

        // Combine them to get the extrusion direction
        let factor = 1.0 + (n1.x * n2.x + n1.y * n2.y);
        let normal = (n1 + n2) / factor;

        // Update the outline points
        outline_vertices[i * 2].position = p1;
        outline_vertices[i * 2 + 1].position = p1 + normal * thickness;
    }

    // Duplicate the first point at the end, to close the outline
    outline_vertices[count * 2].position = outline_vertices[0].position;
    outline_vertices[count * 2 + 1].position = outline_vertices[1].position;

    // Update outline colors
    //updateOutlineColors();

    // Update the shape's bounds
    //m_bounds = m_outlineVertices.getBounds();
}
