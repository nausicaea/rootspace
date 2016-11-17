# -*- coding: utf-8 -*-

# Also check out http://www.lfd.uci.edu/~gohlke/code/transformations.py.html

import ctypes
import math
import sys
import os

# Add OpenGL to the library path, so that PyOpenGL can find it
if sys.platform == "linux" and os.path.isdir("/run/opengl-driver"):
    os.environ["LIBRARY_PATH"] = "/run/opengl-driver/lib:/run/opengl-driver-32/lib:{}".format(os.environ.get("LIBRARY_PATH", ""))

import OpenGL.GL as GL
import glfw
import numpy
from OpenGL.GL.shaders import compileProgram, compileShader


def mat4x4_identity():
    return numpy.eye(4)


def mat4x4_rotation_z(angle):
    s = math.sin(angle)
    c = math.cos(angle)
    Q = (
        (c, -s, 0, 0),
        (s, c, 0, 0),
        (0, 0, 1, 0),
        (0, 0, 0, 1)
    )

    return numpy.array(Q)


def mat4x4_ortho(left, right, bottom, top, near, far):
    l = left
    r = right
    b = bottom
    t = top
    n = near
    f = far
    P = (
        (2 / (r - l), 0, 0, -(r + l) / (r - l)),
        (0, 2 / (t - b), 0, -(t + b) / (t - b)),
        (0, 0, -2 / (f - n), -(f + n) / (f - n)),
        (0, 0, 0, 1)
    )

    return numpy.array(P)


def main():
    # Initialize the library
    if not glfw.init():
        return

    # version hints
    glfw.window_hint(glfw.CONTEXT_VERSION_MAJOR, 3)
    glfw.window_hint(glfw.CONTEXT_VERSION_MINOR, 3)
    glfw.window_hint(glfw.OPENGL_FORWARD_COMPAT, True)
    glfw.window_hint(glfw.OPENGL_PROFILE, glfw.OPENGL_CORE_PROFILE)

    # Create a windowed mode window and its OpenGL context
    window = glfw.create_window(640, 480, "Hello World", None, None)
    if not window:
        glfw.terminate()
        return

    # Make the window's context current
    glfw.make_context_current(window)
    glfw.swap_interval(1)

    # Define the vertices
    vertex_pos = numpy.array([
        -0.6, -0.4, 0.0, 1.0,
        0.6, -0.4, 0.0, 1.0,
        0, 0.6, 0.0, 1.0,
        1.0, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
    ], dtype=numpy.float32)
    num_vertices = 3

    # Define the shaders
    vertex_shader = """
    #version 330 core

    layout(location = 0) in vec4 vPos;
    layout(location = 1) in vec4 vCol;

    uniform mat4 mvp_matrix;

    smooth out vec4 color;

    void main() {


        gl_Position = mvp_matrix * vPos;
        color = vCol;
    }
    """

    fragment_shader = """
    #version 330 core

    smooth in vec4 color;

    out vec4 fragColor;

    void main() {
        fragColor = color;
    }
    """

    # Create and bind the vertex attribute array
    vao = GL.glGenVertexArrays(1)
    GL.glBindVertexArray(vao)

    # Initialize the shaders
    shader_program = compileProgram(
        compileShader(vertex_shader, GL.GL_VERTEX_SHADER),
        compileShader(fragment_shader, GL.GL_FRAGMENT_SHADER)
    )

    # Get the shader parameter locations
    mvp_loc = GL.glGetUniformLocation(shader_program, "mvp_matrix")
    vpos_loc = GL.glGetAttribLocation(shader_program, "vPos")
    vcol_loc = GL.glGetAttribLocation(shader_program, "vCol")

    # Initialize the vertex buffer
    vbo_pos = GL.glGenBuffers(1)
    GL.glBindBuffer(GL.GL_ARRAY_BUFFER, vbo_pos)
    GL.glBufferData(
        GL.GL_ARRAY_BUFFER, vertex_pos.nbytes, vertex_pos, GL.GL_STATIC_DRAW
    )

    # Bind the vertex array
    GL.glEnableVertexAttribArray(vpos_loc)
    GL.glVertexAttribPointer(
        vpos_loc, 4, GL.GL_FLOAT, False, 0, None
    )
    GL.glEnableVertexAttribArray(vcol_loc)
    GL.glVertexAttribPointer(
        vcol_loc, 4, GL.GL_FLOAT, False, 0, ctypes.c_void_p(vertex_pos.nbytes // 2)
    )

    GL.glBindBuffer(GL.GL_ARRAY_BUFFER, 0)
    GL.glBindVertexArray(0)

    # Set the clear color
    GL.glClearColor(0, 0, 0, 1)

    # Loop until the user closes the window
    while not glfw.window_should_close(window):
        width, height = glfw.get_framebuffer_size(window)
        ratio = width / height

        # Calculate the model-view-projection matrix
        m = mat4x4_rotation_z(glfw.get_time())
        v = mat4x4_identity()
        p = mat4x4_ortho(-ratio, ratio, -1, 1, 1, -1)

        # Update the viewport size
        GL.glViewport(0, 0, width, height)
        GL.glClear(GL.GL_COLOR_BUFFER_BIT)

        # Bind the programs and buffers
        GL.glUseProgram(shader_program)
        GL.glBindVertexArray(vao)

        GL.glUniformMatrix4fv(mvp_loc, 1, True, p @ v @ m)

        GL.glDrawArrays(GL.GL_TRIANGLES, 0, num_vertices)

        # Unbind the programs and buffers
        GL.glBindVertexArray(0)
        GL.glUseProgram(0)

        # Swap front and back buffers
        glfw.swap_buffers(window)

        # Poll for and process events
        glfw.poll_events()

    glfw.destroy_window(window)
    glfw.terminate()


if __name__ == "__main__":
    main()