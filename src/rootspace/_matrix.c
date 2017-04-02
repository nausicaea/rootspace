#include "_matrix.h"
#include "_index_handling.h"

const char Matrix_Docstring[] =
    "The constructor accepts a shape parameter, and optionally \n"
    "a data and transposition parameter. The data parameter must be either \n"
    "None, an integer, a floating point number, or a sequence. Raises a \n"
    "TypeError otherwise. Also, if data is a sequence, its length must equal \n"
    "the product of the shape. Raises a ValueError otherwise. Raises a \n"
    "ValueError if the two-dimensional shape is not larger or equal to (1, 1).";

Matrix* Matrix_NewInternal(Py_ssize_t N, Py_ssize_t M, int transposed) {
    Matrix* matrix = PyObject_New(Matrix, &MatrixType);
    if (matrix == NULL) {
        return NULL;
    }

    matrix->container = MatrixContainer_NewInternal(N * M);
    if (matrix->container == NULL) {
        Py_DECREF(matrix);
        return NULL;
    }

    matrix->N = N;
    matrix->M = M;
    matrix->transposed = transposed;

    return matrix;
}

static PyObject* Matrix_New(PyTypeObject* type, PyObject* args, PyObject* kwargs) {
    Py_ssize_t N = 1;
    Py_ssize_t M = 1;
    PyObject* data = NULL;
    int transposed = 0;
    static char* kwlist[] = {"", "data", "transposed", NULL};
    if (!PyArg_ParseTupleAndKeywords(args, kwargs, "(ll)|O$p", kwlist, &N, &M, &data, &transposed)) {
        return NULL;
    }

    if (N == 0 || M == 0) {
        PyErr_SetString(PyExc_ValueError, "Expected the parameter 'shape' to be larger or equal to (1, 1).");
        return NULL;
    }

    Matrix* self = Matrix_NewInternal(N, M, transposed);
    if (self == NULL) {
        return NULL;
    }

    if (data == NULL) {
        for (Py_ssize_t idx = 0; idx < Matrix_SIZE(self); idx++) {
            Matrix_DATA(self)[idx] = (MatrixDataType) 0;
        }
    } else if (PyLong_Check(data)) {
        for (Py_ssize_t idx = 0; idx < Matrix_SIZE(self); idx++) {
            Matrix_DATA(self)[idx] = (MatrixDataType) PyLong_AsLong(data);
        }
    } else if (PyFloat_Check(data)) {
        for (Py_ssize_t idx = 0; idx < Matrix_SIZE(self); idx++) {
            Matrix_DATA(self)[idx] = (MatrixDataType) PyFloat_AsDouble(data);
        }
    } else if (PySequence_Check(data)) {
        if (PySequence_Size(data) != Matrix_SIZE(self)) {
            Py_DECREF(self);
            PyErr_SetString(PyExc_ValueError, "The number of elements in parameter 'data' must correspond to the shape!");
            return NULL;
        }
        for (Py_ssize_t idx = 0; idx < Matrix_SIZE(self); idx++) {
            PyObject* item = PySequence_GetItem(data, idx);
            if (PyLong_Check(item)) {
                Matrix_DATA(self)[idx] = (MatrixDataType) PyLong_AsLong(item);
            } else if (PyFloat_Check(item)) {
                Matrix_DATA(self)[idx] = (MatrixDataType) PyFloat_AsDouble(item);
            } else {
                Py_DECREF(self);
                PyErr_SetString(PyExc_TypeError, "Expected elements of the sequence to be either integers or floats.");
                return NULL;
            }
        }
    } else {
        Py_DECREF(self);
        PyErr_SetString(PyExc_TypeError, "Expected data to be either an integer, a float or a sequence.");
        return NULL;
    }

    return (PyObject*) self;
}

static void Matrix_Dealloc(Matrix* self) {
    Py_XDECREF(self->container);
    Py_TYPE(self)->tp_free((PyObject*) self);
}

static PyObject* Matrix_LessThan(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 1;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (!(Matrix_DATA(fm)[f_element_value] < Matrix_DATA(sm)[s_element_value])) {
                    comp = 0;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            PyErr_SetString(PyExc_ValueError, "Matrices cannot be compared due to a shape mismatch.");
            return NULL;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 1;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] < second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 1;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] < second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value < Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value < Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_LessOrEqual(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 1;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (!(Matrix_DATA(fm)[f_element_value] <= Matrix_DATA(sm)[s_element_value])) {
                    comp = 0;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            PyErr_SetString(PyExc_ValueError, "Matrices cannot be compared due to a shape mismatch.");
            return NULL;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 1;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] <= second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 1;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] <= second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value <= Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value <= Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_Equal(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 1;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (!(Matrix_DATA(fm)[f_element_value] == Matrix_DATA(sm)[s_element_value])) {
                    comp = 0;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 1;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] == second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 1;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] == second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value == Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value == Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_NotEqual(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 0;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (Matrix_DATA(fm)[f_element_value] != Matrix_DATA(sm)[s_element_value]) {
                    comp = 1;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            result = Py_True;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 0;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (Matrix_DATA(first)[i] != second_value) {
                comp = 1;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 0;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (Matrix_DATA(first)[i] != second_value) {
                comp = 1;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 0;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (first_value != Matrix_DATA(second)[i]) {
                comp = 1;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 0;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (first_value != Matrix_DATA(second)[i]) {
                comp = 1;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_GreaterOrEqual(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 1;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (!(Matrix_DATA(fm)[f_element_value] >= Matrix_DATA(sm)[s_element_value])) {
                    comp = 0;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            PyErr_SetString(PyExc_ValueError, "Matrices cannot be compared due to a shape mismatch.");
            return NULL;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 1;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] >= second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 1;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] >= second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value >= Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value >= Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_GreaterThan(PyObject* first, PyObject* second) {
    PyObject* result = Py_NotImplemented;

    if (Matrix_Check(first) && Matrix_Check(second)) {
        if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
            Matrix* fm = (Matrix*) first;
            Matrix* sm = (Matrix*) second;

            PyObject* flinidx = select_all(fm->N, fm->M, fm->transposed);
            if (flinidx == NULL) {
                return NULL;
            }

            PyObject* slinidx = select_all(sm->N, sm->M, sm->transposed);
            if (slinidx == NULL) {
                Py_DECREF(flinidx);
                return NULL;
            }

            int comp = 1;
            for (Py_ssize_t i = 0; i < PyTuple_Size(flinidx); i++) {
                PyObject* f_element = PyTuple_GetItem(flinidx, i);
                Py_ssize_t f_element_value = PyLong_AsLong(f_element);
                PyObject* s_element = PyTuple_GetItem(slinidx, i);
                Py_ssize_t s_element_value = PyLong_AsLong(s_element);
                if (!(Matrix_DATA(fm)[f_element_value] > Matrix_DATA(sm)[s_element_value])) {
                    comp = 0;
                    break;
                }
            }
            Py_DECREF(flinidx);
            Py_DECREF(slinidx);
            if (comp) {
                result = Py_True;
            } else {
                result = Py_False;
            }
        } else {
            PyErr_SetString(PyExc_ValueError, "Matrices cannot be compared due to a shape mismatch.");
            return NULL;
        }
    } else if (Matrix_Check(first) && PyLong_Check(second)) {
        int comp = 1;
        float second_value = (float) PyLong_AsLong(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] > second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (Matrix_Check(first) && PyFloat_Check(second)) {
        int comp = 1;
        float second_value = (float) PyFloat_AsDouble(second);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(first); i++) {
            if (!(Matrix_DATA(first)[i] > second_value)) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyLong_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyLong_AsLong(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value > Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    } else if (PyFloat_Check(first) && Matrix_Check(second)) {
        int comp = 1;
        float first_value = (float) PyFloat_AsDouble(first);
        for (Py_ssize_t i = 0; i < Matrix_SIZE(second); i++) {
            if (!(first_value > Matrix_DATA(second)[i])) {
                comp = 0;
                break;
            }
        }
        if (comp) {
            result = Py_True;
        } else {
            result = Py_False;
        }
    }

    Py_INCREF(result);
    return result;
}

static PyObject* Matrix_RichCompare(PyObject* first, PyObject* second, int op) {
    switch (op) {
        case Py_LT:
            return Matrix_LessThan(first, second);
        case Py_LE:
            return Matrix_LessOrEqual(first, second);
        case Py_EQ:
            return Matrix_Equal(first, second);
        case Py_NE:
            return Matrix_NotEqual(first, second);
        case Py_GE:
            return Matrix_GreaterOrEqual(first, second);
        case Py_GT:
            return Matrix_GreaterThan(first, second);
        default:
            PyErr_SetString(PyExc_RuntimeError, "Unreachable code reached in Matrix_RichCompare.");
            return NULL;
    }
}

static PyObject* Matrix_ToString(Matrix* self) {
    assert((self->N * self->M) == Matrix_SIZE(self));

    PyObject* rows = PyList_New(Matrix_SHAPE_I(self));
    if (rows == NULL) {
        return NULL;
    }
    for (Py_ssize_t i = 0; i < Matrix_SHAPE_I(self); i++) {
        PyObject* row = PyList_New(Matrix_SHAPE_J(self));
        if (row == NULL) {
            Py_DECREF(rows);
            return NULL;
        }
        for (Py_ssize_t j = 0; j < Matrix_SHAPE_J(self); j++) {
            Py_ssize_t idx = linearize_scalar_indices(self->N, self->M, self->transposed, i, j);
            PyList_SetItem(row, j, PyFloat_FromDouble((double) Matrix_DATA(self)[idx]));
        }
        PyList_SetItem(rows, i, row);
    }
    PyObject* s = PyUnicode_FromFormat("%S", rows);
    Py_DECREF(rows);
    return s;
}

static PyObject* Matrix_ToRepresentation(Matrix* self) {
    PyObject* d = PyTuple_New(Matrix_SIZE(self));
    if (d == NULL) {
        return NULL;
    }
    for (Py_ssize_t idx = 0; idx < Matrix_SIZE(self); idx++) {
        PyTuple_SetItem(d, idx, PyFloat_FromDouble((double) Matrix_DATA(self)[idx]));
    }
    PyObject* s = PyUnicode_FromFormat("Matrix((%u, %u), %R, transposed=%u)", self->N, self->M, d, self->transposed);
    Py_DECREF(d);
    return s;
}

static Py_ssize_t Matrix_Length(Matrix* self) {
    assert((self->N * self->M) == Matrix_SIZE(self));

    return Matrix_SIZE(self);
}

static PyObject* Matrix_GetItem(Matrix* self, PyObject* key) {
    PyObject* idx = complete_indices(key);
    if (idx == NULL) {
        return NULL;
    }

    PyObject* sub_shape = get_sub_shape(self->N, self->M, self->transposed, idx);
    if (sub_shape == NULL) {
        Py_DECREF(idx);
        return NULL;
    }

    PyObject* sub_idx = linearize_indices(self->N, self->M, self->transposed, idx);
    if (sub_idx == NULL) {
        Py_DECREF(sub_shape);
        Py_DECREF(idx);
        return NULL;
    }

    PyObject* sub_N_obj = PyTuple_GetItem(sub_shape, 0);
    PyObject* sub_M_obj = PyTuple_GetItem(sub_shape, 1);
    Py_ssize_t sub_N = PyLong_AsLong(sub_N_obj);
    Py_ssize_t sub_M = PyLong_AsLong(sub_M_obj);

    if (Py_SIZE(sub_idx) > 1) {
        Matrix* sub_matrix = Matrix_NewInternal(sub_N, sub_M, 0);
        if (sub_matrix == NULL) {
            Py_DECREF(sub_shape);
            Py_DECREF(sub_idx);
            Py_DECREF(idx);
            return NULL;
        }

        for (Py_ssize_t i = 0; i < Matrix_SIZE(sub_matrix); i++) {
            PyObject* idx_i_obj = PyTuple_GetItem(sub_idx, i);
            Py_ssize_t idx_i = PyLong_AsLong(idx_i_obj);
            Matrix_DATA(sub_matrix)[i] = Matrix_DATA(self)[idx_i];
        }

        Py_DECREF(sub_shape);
        Py_DECREF(sub_idx);
        Py_DECREF(idx);

        return (PyObject*) sub_matrix;
    } else if (Py_SIZE(sub_idx) == 1) {
        PyObject* idx_i_obj = PyTuple_GetItem(sub_idx, 0);
        Py_ssize_t idx_i = PyLong_AsLong(idx_i_obj);

        Py_DECREF(sub_shape);
        Py_DECREF(sub_idx);
        Py_DECREF(idx);

        return PyFloat_FromDouble(Matrix_DATA(self)[idx_i]);
    } else {
        Py_DECREF(sub_shape);
        Py_DECREF(sub_idx);
        Py_DECREF(idx);
        PyErr_SetString(PyExc_ValueError, "Selection resulted in a zero-length linear index");
        return NULL;
    }
}

static int Matrix_SetItem(Matrix* self, PyObject* key, PyObject* value) {
    PyObject* idx = complete_indices(key);
    if (idx == NULL) {
        return -1;
    }

    PyObject* sub_shape = get_sub_shape(self->N, self->M, self->transposed, idx);
    if (sub_shape == NULL) {
        Py_DECREF(idx);
        return -1;
    }

    PyObject* sub_idx = linearize_indices(self->N, self->M, self->transposed, idx);
    if (sub_idx == NULL) {
        Py_DECREF(sub_shape);
        Py_DECREF(idx);
        return -1;
    }

    PyObject* sub_N_obj = PyTuple_GetItem(sub_shape, 0);
    PyObject* sub_M_obj = PyTuple_GetItem(sub_shape, 1);
    Py_ssize_t sub_N = PyLong_AsLong(sub_N_obj);
    Py_ssize_t sub_M = PyLong_AsLong(sub_M_obj);
    Py_ssize_t sub_length = sub_N * sub_M;

    if (Matrix_Check(value)) {
        Matrix* value_obj = (Matrix*) value;
        if (Matrix_SHAPE_I(value_obj) != sub_N || Matrix_SHAPE_J(value_obj) != sub_M) {
            Py_DECREF(sub_idx);
            Py_DECREF(sub_shape);
            Py_DECREF(idx);
            PyErr_SetString(PyExc_ValueError, "Shape mismatch between indexed range and submitted Matrix value.");
            return -1;
        }

        PyObject* value_idx = select_all(value_obj->N, value_obj->M, value_obj->transposed);

        for (Py_ssize_t i = 0; i < sub_length; i++) {
            PyObject* sub_idx_obj = PyTuple_GetItem(sub_idx, i);
            Py_ssize_t sub_idx_value = PyLong_AsSsize_t(sub_idx_obj);
            PyObject* value_idx_obj = PyTuple_GetItem(value_idx, i);
            Py_ssize_t value_idx_value = PyLong_AsSsize_t(value_idx_obj);
            Matrix_DATA(self)[sub_idx_value] = Matrix_DATA(value_obj)[value_idx_value];
        }
    } else if (PySequence_Check(value)) {
        if (PySequence_Size(value) != sub_length) {
            Py_DECREF(sub_idx);
            Py_DECREF(sub_shape);
            Py_DECREF(idx);
            PyErr_SetString(PyExc_ValueError, "The submitted value does not have the same length as the indexed range.");
            return -1;
        }
        for (Py_ssize_t i = 0; i < sub_length; i++) {
            PyObject* value_obj = PySequence_GetItem(value, i);
            MatrixDataType value_data;
            if (PyFloat_Check(value_obj)) {
                value_data = (MatrixDataType) PyFloat_AsDouble(value_obj);
            } else if (PyLong_Check(value_obj)) {
                value_data = (MatrixDataType) PyLong_AsLong(value_obj);
            } else {
                Py_DECREF(sub_idx);
                Py_DECREF(sub_shape);
                Py_DECREF(idx);
                PyErr_SetString(PyExc_TypeError, "The elements of the submitted Sequence must be integers or floats.");
                return -1;
            }
            PyObject* sub_idx_obj = PyTuple_GetItem(sub_idx, i);
            Py_ssize_t sub_idx_value = PyLong_AsSsize_t(sub_idx_obj);
            Matrix_DATA(self)[sub_idx_value] = value_data;
        }
    } else if (PyLong_Check(value)) {
        MatrixDataType value_data = (MatrixDataType) PyLong_AsLong(value);
        for (Py_ssize_t i = 0; i < sub_length; i++) {
            PyObject* sub_idx_obj = PyTuple_GetItem(sub_idx, i);
            Py_ssize_t sub_idx = PyLong_AsSsize_t(sub_idx_obj);
            Matrix_DATA(self)[sub_idx] = value_data;
        }
    } else if (PyFloat_Check(value)) {
        MatrixDataType value_data = (MatrixDataType) PyFloat_AsDouble(value);
        for (Py_ssize_t i = 0; i < sub_length; i++) {
            PyObject* sub_idx_obj = PyTuple_GetItem(sub_idx, i);
            Py_ssize_t sub_idx = PyLong_AsSsize_t(sub_idx_obj);
            Matrix_DATA(self)[sub_idx] = value_data;
        }
    } else {
        Py_DECREF(sub_idx);
        Py_DECREF(sub_shape);
        Py_DECREF(idx);
        PyErr_SetString(PyExc_TypeError, "Expected a Matrix, a Sequence, an integer or a float as values.");
        return -1;
    }

    return 0;
}

//
// static PyObject* Matrix_Negative(Matrix* self) {
//     Matrix* matrix = Matrix_NewInternal(self->N, self->M, self->transposed);
//     if (matrix == NULL) {
//         return NULL;
//     }
//
//     Py_ssize_t i;
//     for (i = 0; i < Py_SIZE(matrix); i++) {
//         Matrix_DATA(matrix)[i] = -Matrix_DATA(self)[i];
//     }
//
//     return (PyObject*) matrix;
// }
//
// static PyObject* Matrix_Positive(Matrix* self) {
//     Matrix* matrix = Matrix_NewInternal(self->N, self->M, self->transposed);
//     if (matrix == NULL) {
//         return NULL;
//     }
//
//     Py_ssize_t i;
//     for (i = 0; i < Py_SIZE(matrix); i++) {
//         Matrix_DATA(matrix)[i] = Matrix_DATA(self)[i];
//     }
//
//     return (PyObject*) matrix;
// }
//
// static PyObject* Matrix_Absolute(Matrix* self) {
//     Matrix* matrix = Matrix_NewInternal(self->N, self->M, self->transposed);
//     if (matrix == NULL) {
//         return NULL;
//     }
//
//     Py_ssize_t i;
//     for (i = 0; i < Py_SIZE(matrix); i++) {
//         Matrix_DATA(matrix)[i] = (float) fabs(Matrix_DATA(self)[i]);
//     }
//
//     return (PyObject*) matrix;
// }
//
// static PyObject* Matrix_Add(PyObject* first, PyObject* second) {
//     if (Matrix_Check(first) && Matrix_Check(second)) {
//         if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
//             Matrix* fm = (Matrix*) first;
//             Matrix* sm = (Matrix*) second;
//
//             PyObject* flinidx = select_all(fm);
//             if (flinidx == NULL) {
//                 return NULL;
//             }
//
//             PyObject* slinidx = select_all(sm);
//             if (slinidx == NULL) {
//                 Py_DECREF(flinidx);
//                 return NULL;
//             }
//
//             Matrix* result = Matrix_NewInternal(fm->N, fm->M, fm->transposed);
//
//             Py_ssize_t i;
//             for (i = 0; i < PyTuple_Size(flinidx); i++) {
//                 PyObject* f_element = PyTuple_GetItem(flinidx, i);
//                 Py_ssize_t f_element_value = PyLong_AsLong(f_element);
//                 PyObject* s_element = PyTuple_GetItem(slinidx, i);
//                 Py_ssize_t s_element_value = PyLong_AsLong(s_element);
//                 Matrix_DATA(result)[f_element_value] = Matrix_DATA(fm)[f_element_value] + Matrix_DATA(sm)[s_element_value];
//             }
//             Py_DECREF(flinidx);
//             Py_DECREF(slinidx);
//             return (PyObject*) result;
//         } else {
//             PyErr_SetString(PyExc_ValueError, "Cannot perform addition on Matrices of differing shapes.");
//             return NULL;
//         }
//     } else if (Matrix_Check(first) && PyLong_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyLong_AsLong(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] + second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (Matrix_Check(first) && PyFloat_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyFloat_AsDouble(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] + second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (PyLong_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyLong_AsLong(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value + Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else if (PyFloat_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyFloat_AsDouble(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value + Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else {
//         PyObject* result = Py_NotImplemented;
//         Py_INCREF(result);
//         return result;
//     }
// }
//
// static PyObject* Matrix_Subtract(PyObject* first, PyObject* second) {
//     if (Matrix_Check(first) && Matrix_Check(second)) {
//         if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
//             Matrix* fm = (Matrix*) first;
//             Matrix* sm = (Matrix*) second;
//
//             PyObject* flinidx = select_all(fm);
//             if (flinidx == NULL) {
//                 return NULL;
//             }
//
//             PyObject* slinidx = select_all(sm);
//             if (slinidx == NULL) {
//                 Py_DECREF(flinidx);
//                 return NULL;
//             }
//
//             Matrix* result = Matrix_NewInternal(fm->N, fm->M, fm->transposed);
//
//             Py_ssize_t i;
//             for (i = 0; i < PyTuple_Size(flinidx); i++) {
//                 PyObject* f_element = PyTuple_GetItem(flinidx, i);
//                 Py_ssize_t f_element_value = PyLong_AsLong(f_element);
//                 PyObject* s_element = PyTuple_GetItem(slinidx, i);
//                 Py_ssize_t s_element_value = PyLong_AsLong(s_element);
//                 Matrix_DATA(result)[f_element_value] = Matrix_DATA(fm)[f_element_value] - Matrix_DATA(sm)[s_element_value];
//             }
//             Py_DECREF(flinidx);
//             Py_DECREF(slinidx);
//             return (PyObject*) result;
//         } else {
//             PyErr_SetString(PyExc_ValueError, "Cannot perform addition on Matrices of differing shapes.");
//             return NULL;
//         }
//     } else if (Matrix_Check(first) && PyLong_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyLong_AsLong(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] - second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (Matrix_Check(first) && PyFloat_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyFloat_AsDouble(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] - second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (PyLong_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyLong_AsLong(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value - Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else if (PyFloat_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyFloat_AsDouble(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value - Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else {
//         PyObject* result = Py_NotImplemented;
//         Py_INCREF(result);
//         return result;
//     }
// }
//
// static PyObject* Matrix_Multiply(PyObject* first, PyObject* second) {
//     if (Matrix_Check(first) && Matrix_Check(second)) {
//         if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
//             Matrix* fm = (Matrix*) first;
//             Matrix* sm = (Matrix*) second;
//
//             PyObject* flinidx = select_all(fm);
//             if (flinidx == NULL) {
//                 return NULL;
//             }
//
//             PyObject* slinidx = select_all(sm);
//             if (slinidx == NULL) {
//                 Py_DECREF(flinidx);
//                 return NULL;
//             }
//
//             Matrix* result = Matrix_NewInternal(fm->N, fm->M, fm->transposed);
//
//             Py_ssize_t i;
//             for (i = 0; i < PyTuple_Size(flinidx); i++) {
//                 PyObject* f_element = PyTuple_GetItem(flinidx, i);
//                 Py_ssize_t f_element_value = PyLong_AsLong(f_element);
//                 PyObject* s_element = PyTuple_GetItem(slinidx, i);
//                 Py_ssize_t s_element_value = PyLong_AsLong(s_element);
//                 Matrix_DATA(result)[f_element_value] = Matrix_DATA(fm)[f_element_value] * Matrix_DATA(sm)[s_element_value];
//             }
//             Py_DECREF(flinidx);
//             Py_DECREF(slinidx);
//             return (PyObject*) result;
//         } else {
//             PyErr_SetString(PyExc_ValueError, "Cannot perform addition on Matrices of differing shapes.");
//             return NULL;
//         }
//     } else if (Matrix_Check(first) && PyLong_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyLong_AsLong(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] * second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (Matrix_Check(first) && PyFloat_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyFloat_AsDouble(second);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] * second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (PyLong_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyLong_AsLong(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value * Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else if (PyFloat_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyFloat_AsDouble(first);
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value * Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else {
//         PyObject* result = Py_NotImplemented;
//         Py_INCREF(result);
//         return result;
//     }
// }
//
// static PyObject* Matrix_TrueDivide(PyObject* first, PyObject* second) {
//     if (Matrix_Check(first) && Matrix_Check(second)) {
//         if (Matrix_SHAPE_I(first) == Matrix_SHAPE_I(second) && Matrix_SHAPE_J(first) == Matrix_SHAPE_J(second)) {
//             Matrix* fm = (Matrix*) first;
//             Matrix* sm = (Matrix*) second;
//
//             PyObject* flinidx = select_all(fm);
//             if (flinidx == NULL) {
//                 return NULL;
//             }
//
//             PyObject* slinidx = select_all(sm);
//             if (slinidx == NULL) {
//                 Py_DECREF(flinidx);
//                 return NULL;
//             }
//
//             Matrix* result = Matrix_NewInternal(fm->N, fm->M, fm->transposed);
//
//             Py_ssize_t i;
//             for (i = 0; i < PyTuple_Size(flinidx); i++) {
//                 PyObject* f_element = PyTuple_GetItem(flinidx, i);
//                 Py_ssize_t f_element_value = PyLong_AsLong(f_element);
//                 PyObject* s_element = PyTuple_GetItem(slinidx, i);
//                 Py_ssize_t s_element_value = PyLong_AsLong(s_element);
//                 float s_value = Matrix_DATA(sm)[s_element_value];
//                 if (s_value != 0.0f) {
//                     Matrix_DATA(result)[f_element_value] = Matrix_DATA(fm)[f_element_value] / s_value;
//                 } else {
//                     Py_DECREF(flinidx);
//                     Py_DECREF(slinidx);
//                     Py_DECREF(result);
//                     PyErr_SetNone(PyExc_ZeroDivisionError);
//                     return NULL;
//                 }
//             }
//             Py_DECREF(flinidx);
//             Py_DECREF(slinidx);
//             return (PyObject*) result;
//         } else {
//             PyErr_SetString(PyExc_ValueError, "Cannot perform addition on Matrices of differing shapes.");
//             return NULL;
//         }
//     } else if (Matrix_Check(first) && PyLong_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyLong_AsLong(second);
//         if (second_value == 0.0f) {
//             Py_DECREF(result);
//             PyErr_SetNone(PyExc_ZeroDivisionError);
//             return NULL;
//         }
//
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] / second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (Matrix_Check(first) && PyFloat_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(first), Matrix_M(first), Matrix_TRANSPOSED(first));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float second_value = (float) PyFloat_AsDouble(second);
//         if (second_value == 0.0f) {
//             Py_DECREF(result);
//             PyErr_SetNone(PyExc_ZeroDivisionError);
//             return NULL;
//         }
//
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = Matrix_DATA(first)[i] / second_value;
//         }
//
//         return (PyObject*) result;
//     } else if (PyLong_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyLong_AsLong(first);
//         if (first_value == 0.0f) {
//             Py_DECREF(result);
//             PyErr_SetNone(PyExc_ZeroDivisionError);
//             return NULL;
//         }
//
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value / Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else if (PyFloat_Check(first) && Matrix_Check(second)) {
//         Matrix* result = Matrix_NewInternal(Matrix_N(second), Matrix_M(second), Matrix_TRANSPOSED(second));
//         if (result == NULL) {
//             return NULL;
//         }
//
//         float first_value = (float) PyFloat_AsDouble(first);
//         if (first_value == 0.0f) {
//             Py_DECREF(result);
//             PyErr_SetNone(PyExc_ZeroDivisionError);
//             return NULL;
//         }
//
//         Py_ssize_t i;
//         for (i = 0; i < Py_SIZE(result); i++) {
//             Matrix_DATA(result)[i] = first_value / Matrix_DATA(second)[i];
//         }
//
//         return (PyObject*) result;
//     } else {
//         PyObject* result = Py_NotImplemented;
//         Py_INCREF(result);
//         return result;
//     }
// }

// static PyObject* Matrix_MatMultiply(PyObject* first, PyObject* second) {
//     if (Matrix_Check(first) && Matrix_Check(second)) {
//         if (Matrix_SHAPE_J(first) == Matrix_SHAPE_I(second)) {
//             Matrix* fm = (Matrix*) first;
//             Matrix* sm = (Matrix*) second;
//             Py_ssize_t N = Matrix_SHAPE_I(first);
//             Py_ssize_t shape_m = Matrix_SHAPE_J(first);
//             Py_ssize_t M = Matrix_SHAPE_J(second);
//             if (N == 1 && M == 1) {
//                 float result = 0.0f;
//                 Py_ssize_t m;
//                 for (m = 0; m < shape_m; m++) {
//                     result += Matrix_DATA(fm)[m] * Matrix_DATA(sm)[m];
//                 }
//                 return PyFloat_FromDouble((double) result);
//             } else {
//                 Matrix* result = Matrix_NewInternal(N, M, 0);
//                 if (result == NULL) {
//                     return NULL;
//                 }
//                 Py_ssize_t i;
//                 for (i = 0; i < N; i++) {
//                     Py_ssize_t j;
//                     for (j = 0; j < M; j++) {
//                         float temp = 0.0f;
//                         Py_ssize_t m;
//                         for (m = 0; m < shape_m; m++) {
//                             Py_ssize_t flinidx = linearize_scalar_indices(N, shape_m, i, m);
//                             Py_ssize_t slinidx = linearize_scalar_indices(shape_m, M, m, j);
//                             temp += Matrix_DATA(fm)[flinidx] * Matrix_DATA(sm)[slinidx];
//                         }
//                         Py_ssize_t rlinidx = linearize_scalar_indices(N, M, i, j);
//                         Matrix_DATA(result)[rlinidx] = temp;
//                     }
//                 }
//                 return (PyObject*) result;
//             }
//         } else {
//             PyErr_SetString(PyExc_ValueError, "Shape mismatch; the last dimension of the first and the first dimension of the second operand must be equal.");
//             return NULL;
//         }
//     } else {
//         PyObject* result = Py_NotImplemented;
//         Py_INCREF(result);
//         return result;
//     }
// }
//
// static PyNumberMethods MatrixAsNumber = {
//     (binaryfunc) Matrix_Add,
//     (binaryfunc) Matrix_Subtract,
//     (binaryfunc) Matrix_Multiply,
//     0,  // Matrix_Remainder
//     0,  // Matrix_Divmod
//     0,  // Matrix_Power
//     (unaryfunc) Matrix_Negative,
//     (unaryfunc) Matrix_Positive,
//     (unaryfunc) Matrix_Absolute,
//     0,  // Matrix_Bool
//     0,  // Matrix_Invert
//     0,  // Matrix_LeftShift
//     0,  // Matrix_RightShift
//     0,  // Matrix_And
//     0,  // Matrix_Xor
//     0,  // Matrix_Or
//     0,  // Matrix_Int
//     0,  // Reserved
//     0,  // Matrix_Float
//
//     0,  // Matrix_InplaceAdd
//     0,  // Matrix_InplaceSubtract
//     0,  // Matrix_InplaceMultiply
//     0,  // Matrix_InplaceRemainder
//     0,  // Matrix_InplacePower
//     0,  // Matrix_InplaceLeftShift
//     0,  // Matrix_InplaceRightShift
//     0,  // Matrix_InplaceAnd
//     0,  // Matrix_InplaceXor
//     0,  // Matrix_InplaceOr
//
//     0,  // Matrix_FloorDivide
//     (binaryfunc) Matrix_TrueDivide,
//     0,  // Matrix_InplaceFloorDivide
//     0,  // Matrix_InplaceTrueDivide
//
//     0,  // Matrix_Index
//
//     (binaryfunc) Matrix_MatMultiply,
//     0,  // Matrix_InplaceMatMultiply
// };

static PyMappingMethods MatrixAsMapping = {
    (lenfunc) Matrix_Length,
    (binaryfunc) Matrix_GetItem,
    (objobjargproc) Matrix_SetItem
};

/// Define the Matrix type object (MatrixType).
PyTypeObject MatrixType = {
    PyVarObject_HEAD_INIT(NULL, 0)
    "_math.Matrix",                            /* tp_name */
    sizeof(Matrix),                           /* tp_basicsize */
    0,                                        /* tp_itemsize */
    (destructor) Matrix_Dealloc,              /* tp_dealloc */
    0,                                        /* tp_print */
    0,                                        /* tp_getattr */
    0,                                        /* tp_setattr */
    0,                                        /* tp_reserved */
    (reprfunc) Matrix_ToRepresentation,       /* tp_repr */
    0, //&MatrixAsNumber,                          /* tp_as_number */
    0,                                        /* tp_as_sequence */
    &MatrixAsMapping,                         /* tp_as_mapping */
    0,                                        /* tp_hash  */
    0,                                        /* tp_call */
    (reprfunc) Matrix_ToString,                    /* tp_str */
    0,                                        /* tp_getattro */
    0,                                        /* tp_setattro */
    0,                                        /* tp_as_buffer */
    Py_TPFLAGS_DEFAULT,                       /* tp_flags */
    Matrix_Docstring,                         /* tp_doc */
    0,                                        /* tp_traverse */
    0,                                        /* tp_clear */
    (richcmpfunc) Matrix_RichCompare,         /* tp_richcompare */
    0,                                        /* tp_weaklistoffset */
    0,                                        /* tp_iter */
    0,                                        /* tp_iternext */
    0,                                        /* tp_methods */
    0,                                        /* tp_members */
    0,                                        /* tp_getset */
    0,                                        /* tp_base */
    0,                                        /* tp_dict */
    0,                                        /* tp_descr_get */
    0,                                        /* tp_descr_set */
    0,                                        /* tp_dictoffset */
    0,                                        /* tp_init */
    0,                                        /* tp_alloc */
    Matrix_New,                               /* tp_new */
    PyObject_Del                              /* tp_free */
};

