from python_extension import PriorityQueue
import unittest


class Test(unittest.TestCase):
    def test_basics(self):
        queue = PriorityQueue()

        # Empty queue behavior
        self.assertEqual(None, queue.peek())
        self.assertTrue(queue.is_empty())
        self.assertEqual(0, len(queue))

        queue.push(3)
        queue.push(5)
        queue.push(1)
        self.assertEqual(3, len(queue))

        # Index
        self.assertEqual(5, queue[0])
        self.assertEqual(3, queue[1])
        self.assertEqual(1, queue[2])


if __name__ == '__main__':
    unittest.main()


