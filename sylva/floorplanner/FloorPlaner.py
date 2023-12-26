#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from ortools.sat.python import cp_model
import math
import Floorplan_pb2
from google.protobuf.json_format import MessageToJson


# Define the FloorPlanner class:
class FloorPlanner:
    """FloorPlanner class."""

    def __init__(self):
        pass

    def floorplan_fast(self, fp: Floorplan_pb2.Floorplan):
        num_squares = len(fp.shape)

        model = cp_model.CpModel()
        x_intervals = []
        y_intervals = []
        x_starts = []
        y_starts = []
        x_end = []
        y_end = []

        # Creates intervals for the NoOverlap2D and size variables.
        for i in range(num_squares):
            start_x = model.NewIntVar(0, fp.max_width, "sx_%i" % i)
            end_x = model.NewIntVar(0, fp.max_width, "ex_%i" % i)
            start_y = model.NewIntVar(0, fp.max_height, "sy_%i" % i)
            end_y = model.NewIntVar(0, fp.max_height, "ey_%i" % i)

            interval_x = model.NewIntervalVar(
                start_x, fp.shape[i].width, end_x, "ix_%i" % i)
            interval_y = model.NewIntervalVar(
                start_y, fp.shape[i].height, end_y, "iy_%i" % i)

            x_intervals.append(interval_x)
            y_intervals.append(interval_y)
            x_starts.append(start_x)
            y_starts.append(start_y)
            x_end.append(end_x)
            y_end.append(end_y)

        # Main constraint.
        model.AddNoOverlap2D(x_intervals, y_intervals)

        # Symmetry breaking 2: first square in one quadrant.
        model.Add(x_starts[0] < (fp.max_width + 1) // 2)
        model.Add(y_starts[0] < (fp.max_height + 1) // 2)

        # Compute the maximum x and y positions.
        max_x_position = model.NewIntVar(0, fp.max_width, "max_x_position")
        model.AddMaxEquality(max_x_position, [x for x in x_end])
        max_y_position = model.NewIntVar(0, fp.max_height, "max_y_position")
        model.AddMaxEquality(max_y_position, [y for y in y_end])

        # Compute the total area=max_x_position*max_y_position.
        total_area = model.NewIntVar(
            0, fp.max_width * fp.max_height, "total_area")
        model.AddMultiplicationEquality(
            total_area, [max_x_position, max_y_position])

        model.Minimize(total_area)

        # Creates a solver and solves.
        solver = cp_model.CpSolver()
        solver.parameters.num_workers = 16
        # solver.parameters.log_search_progress = True
        solver.parameters.max_time_in_seconds = 10.0
        status = solver.Solve(model)
        solution_found = status == cp_model.OPTIMAL or status == cp_model.FEASIBLE

        if solution_found:
            fp.pos.clear()
            for i in range(num_squares):
                p = Floorplan_pb2.RectanglePosion()
                p.x = solver.Value(x_starts[i])
                p.y = solver.Value(y_starts[i])
                fp.pos.append(p)
            fp.max_width = solver.Value(max_x_position)
            fp.max_height = solver.Value(max_y_position)

        return [solution_found, fp]

    def floorplan_detail(self, fp: Floorplan_pb2.Floorplan):
        x_sizes = [r.width for r in fp.shape]
        y_sizes = [r.height for r in fp.shape]
        num_squares = len(x_sizes)

        """Try to fill the rectangle with a given number of squares."""
        size_x = fp.max_width
        size_y = fp.max_height

        model = cp_model.CpModel()

        areas = []
        x_intervals = []
        y_intervals = []
        x_starts = []
        y_starts = []

        # Creates intervals for the NoOverlap2D and size variables.
        for i in range(num_squares):
            start_x = model.NewIntVar(0, size_x, "sx_%i" % i)
            end_x = model.NewIntVar(0, size_x, "ex_%i" % i)
            start_y = model.NewIntVar(0, size_y, "sy_%i" % i)
            end_y = model.NewIntVar(0, size_y, "ey_%i" % i)

            interval_x = model.NewIntervalVar(
                start_x, x_sizes[i], end_x, "ix_%i" % i)
            interval_y = model.NewIntervalVar(
                start_y, y_sizes[i], end_y, "iy_%i" % i)

            area = x_sizes[i] * y_sizes[i]
            areas.append(area)
            x_intervals.append(interval_x)
            y_intervals.append(interval_y)
            x_starts.append(start_x)
            y_starts.append(start_y)

        # Main constraint.
        model.AddNoOverlap2D(x_intervals, y_intervals)

        # Symmetry breaking 2: first square in one quadrant.
        model.Add(x_starts[0] < (size_x + 1) // 2)
        model.Add(y_starts[0] < (size_y + 1) // 2)

        # Symmetry breaking 3: minimal value in start_x equals to zero and minimal value in start_y equals to zero.
        model.AddMinEquality(0, x_starts)
        model.AddMinEquality(0, y_starts)

        # Compute the distance matrix between each pair of squares. The distance is Manhattan distance from the center of square 1 to the center of square 2.
        distance_matrix = []
        index = 0
        for i in range(num_squares):
            for j in range(i + 1, num_squares):
                distance_matrix.append(model.NewIntVar(
                    0, size_x + size_y, "dist_%i_%i" % (i, j)))
                dx_0 = model.NewIntVar(-size_x, size_x, "dx0_%i_%i" % (i, j))
                model.Add(dx_0 == x_starts[i] + x_sizes[i] //
                          2 - x_starts[j] - x_sizes[j] // 2)
                dx = model.NewIntVar(0, size_x, "dx_%i_%i" % (i, j))
                model.AddAbsEquality(dx, dx_0)
                dy_0 = model.NewIntVar(-size_y, size_y, "dy0_%i_%i" % (i, j))
                model.Add(dy_0 == y_starts[i] + y_sizes[i] //
                          2 - y_starts[j] - y_sizes[j] // 2)
                dy = model.NewIntVar(0, size_y, "dy_%i_%i" % (i, j))
                model.AddAbsEquality(dy, dy_0)
                model.Add(distance_matrix[index] == dx + dy)
                index += 1

        # Compute the weighted distance by multiplying the distance matrix by the connectivity matrix.
        weighted_distance = []
        index = 0
        for i in range(num_squares):
            for j in range(i + 1, num_squares):
                weighted_distance.append(model.NewIntVar(
                    0, 10000, "wdist_%i_%i" % (i, j)))
                model.AddMultiplicationEquality(weighted_distance[index],
                                                [distance_matrix[index], fp.conn[index]])
                index += 1

        # Compute the total weighted distance.
        total_weighted_distance = model.NewIntVar(
            0, 10000, "total_weighted_distance")
        model.Add(sum(weighted_distance) == total_weighted_distance)

        # Objective: minimize the total weighted distance.
        model.Minimize(total_weighted_distance)

        # Compute the maximum x and y positions.
        max_x_position = model.NewIntVar(0, size_x, "max_x_position")
        model.AddMaxEquality(
            max_x_position, [interval.EndExpr() for interval in x_intervals])
        max_y_position = model.NewIntVar(0, size_y, "max_y_position")
        model.AddMaxEquality(
            max_y_position, [interval.EndExpr() for interval in y_intervals])

        # Creates a solver and solves.
        solver = cp_model.CpSolver()
        solver.parameters.num_workers = 16
        # solver.parameters.log_search_progress = True
        solver.parameters.max_time_in_seconds = 10.0
        status = solver.Solve(model)
        solution_found = status == cp_model.OPTIMAL or status == cp_model.FEASIBLE
        if solution_found:
            fp.pos.clear()
            for i in range(num_squares):
                p = Floorplan_pb2.RectanglePosion()
                p.x = solver.Value(x_starts[i])
                p.y = solver.Value(y_starts[i])
                fp.pos.append(p)

            fp.max_width = solver.Value(max_x_position)
            fp.max_height = solver.Value(max_y_position)
        return [solution_found, fp]

    # This function generate a picture of the floorplan. It accepts four parameters:
    # - max_x: the maximum x position of the floorplan
    # - max_y: the maximum y position of the floorplan
    # - start: the start position of each rectangle. Each rectangle is a list of two elements: the x position and the y position
    # - size: the size of each rectangles. Each rectangle is a list of two elements: the width and the height of the rectangle
    # In the generated picture, each rectangle is represented by a different light color. The color is generated randomly. The grid is also shown in the picture as dashed line.
    # It also mark the index of each rectangle in bold font in the center of each drawed rectangle. The color of the font is the opposite of the color of the rectangle.

    def generate_picture(self, filename: str, fp: Floorplan_pb2.Floorplan) -> None:
        import matplotlib.pyplot as plt
        import matplotlib.patches as patches
        import random
        max_x = fp.max_width
        max_y = fp.max_height
        start = fp.pos
        size = fp.shape
        fig = plt.figure()
        ax = fig.add_subplot(111, aspect='equal')
        ax.set_xlim([0, max_x])
        ax.set_ylim([0, max_y])
        for i in range(len(start)):
            x = random.random()
            y = random.random()
            z = random.random()
            ax.add_patch(patches.Rectangle(
                (start[i].x, start[i].y), size[i].width, size[i].height, color=(x, y, z)))
            ax.text(start[i].x + size[i].width / 2, start[i].y + size[i].height / 2, str(
                i), horizontalalignment='center', verticalalignment='center', weight='bold', color=(1 - x, 1 - y, 1 - z))
        plt.grid(True, linestyle='--')
        # save the picture to pdf file
        plt.savefig(filename, bbox_inches='tight')

    # This function generates random floorplan data. It accepts one parameter:
    # - num_rectangles: the number of rectangles in the floorplan
    # It returns the Floorplan object.

    def generate_random_floorplan(self, num_rectangles):
        import random
        fp = Floorplan_pb2.Floorplan()
        fp.max_width = 100
        fp.max_height = 100
        for i in range(num_rectangles):
            shape = fp.shape.add()
            shape.width = random.randint(1, 10)
            shape.height = random.randint(1, 10)
        for i in range(num_rectangles):
            for j in range(i + 1, num_rectangles):
                fp.conn.append(random.randint(0, 10))
        return fp

    def run_pb(self, fp: Floorplan_pb2.Floorplan) -> [bool, Floorplan_pb2.Floorplan]:
        constraint_x = fp.max_width
        constraint_y = fp.max_height

        # call the floorplan_fast function to generate the floorplan
        [status, fp] = self.floorplan_fast(fp)
        if not status:
            print("No solution found")
            return [False, fp]

        # change the fp object to restrict the max_width/max_height to be 1.1 times the max_width/max_height returned by the floorplan_fast function
        fp.max_width = int(math.ceil(fp.max_width*1.1))
        fp.max_height = int(math.ceil(fp.max_height*1.1))
        if fp.max_width > constraint_x:
            fp.max_width = constraint_x
        if fp.max_height > constraint_y:
            fp.max_height = constraint_y

        # call the floorplan_detail function to generate the floorplan
        [status, fp] = self.floorplan_detail(fp)
        if not status:
            print("No solution found")
            return [False, fp]

        return [True, fp]

    # This function is the main function of the program. It accepts two parameters:
    # - input_file: the name of the file that contains the floorplan data in protobuf format
    # - output_dir: the name of the output directory.
    # It generates three files in the output directory:
    # - floorplan.pdf: the picture of the floorplan
    # - floorplan.json: the text file that contains the floorplan data converted to json format
    # - floorplan.bin: the binary file that contains the floorplan data in protobuf format

    def run_file(self, input_file: str, output_dir: str) -> None:
        # read the floorplan data from the input file
        fp = Floorplan_pb2.Floorplan()
        with open(input_file, "rb") as f:
            fp.ParseFromString(f.read())
        [status, fp] = self.run_pb(fp)

        if status:
            # generate the picture of the floorplan
            self.generate_picture(output_dir+"/floorplan.pdf", fp)

            # write the floorplan data to the output directory
            with open(output_dir+"/floorplan.bin", "wb") as f:
                f.write(fp.SerializeToString())

            # write the floorplan data to the output directory in json format
            with open(output_dir+"/floorplan.json", "w") as f:
                f.write(MessageToJson(fp))


if __name__ == "__main__":
    fpr = FloorPlanner()
    fp = fpr.generate_random_floorplan(3)
    with open("floorplan_input.bin", "wb") as f:
        f.write(fp.SerializeToString())
    fpr.run_file("floorplan_input.bin", ".")
