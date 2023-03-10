from python_extension import PriorityQueue as PQueue
import unittest


class Test(unittest.TestCase):
    def test_indexing(self):
        self.queue.push(5)
        self.queue.push(7)
        self.queue.push(3)

        self.assertEqual(7, self.queue[0])
        self.assertEqual(5, self.queue[1])
        self.assertEqual(3, self.queue[2])

    def test_length_tracking(self):
        self.assertEqual(0, len(self.queue))
        self.queue.push(5)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(1)
        self.queue.push(3)
        self.queue.peek()  # Peek does not modify length
        self.assertEqual(5, len(self.queue))
        # self.queue.pop()
        self.queue.clear()
        self.assertEqual(0, len(self.queue))

    def test_contains(self):
        self.assertFalse(69 in self.queue)

        self.queue.push(1)
        self.queue.push(10)
        self.queue.push(5)
        self.assertTrue(5 in self.queue)
        self.assertTrue(10 in self.queue)
        self.assertTrue(1 in self.queue)
        self.queue.push(5)  # Dupes
        self.assertTrue(5 in self.queue)

        self.queue.clear()
        self.assertFalse(5 in self.queue)
        self.assertFalse(10 in self.queue)
        self.assertFalse(1 in self.queue)

    def test_empty_behavior(self):
        self.assertEqual(None, self.queue.peek())
        self.assertTrue(self.queue.is_empty())
        # self.assertEqual(None, self.queue.pop())  # Uncomment when implmented

    def setUp(self) -> None:
        self.queue = PQueue()


if __name__ == '__main__':
    unittest.main()


