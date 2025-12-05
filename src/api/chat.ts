// Simple mock API for chatbot demonstration
// In a real application, this would connect to an actual AI service

export async function POST(request: globalThis.Request) {
  try {
    const { messages } = await request.json()

    if (!messages || !Array.isArray(messages)) {
      return new globalThis.Response('Invalid messages format', { status: 400 })
    }

    // Simple bot responses
    const responses = [
      "That's interesting! Tell me more about that.",
      'I understand. How can I help you with that?',
      'Thanks for sharing! What else would you like to discuss?',
      'I see your point. Let me think about that for a moment.',
      "That's a great question! Here's what I think about it.",
      'I appreciate you sharing that with me.',
      'That makes sense. What are your thoughts on this?',
      'Interesting perspective! Have you considered other angles?',
      "I'd love to help you explore that idea further.",
      "That's a fascinating topic! Let me share what I know about it.",
    ]

    // Simulate processing time
    await new Promise((resolve) => globalThis.setTimeout(resolve, 1000))

    const botResponse = responses[Math.floor(Math.random() * responses.length)]

    return new globalThis.Response(
      JSON.stringify({
        role: 'assistant',
        content: botResponse,
      }),
      {
        headers: {
          'Content-Type': 'application/json',
        },
      }
    )
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error('Chat API error:', error)
    return new globalThis.Response('Internal Server Error', { status: 500 })
  }
}
